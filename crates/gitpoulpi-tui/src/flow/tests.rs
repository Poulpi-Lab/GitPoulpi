//! Buffer tests of the Flow dialog, the grouped sidebar rows, the branch
//! tooltips and the commands, over a fake [`FlowActions`].

use std::sync::Arc;
use std::time::{Duration, Instant};

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers, MouseButton, MouseEvent, MouseEventKind};
use gitpoulpi_core::workflow::WorkflowTemplate;
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;

use super::actions::{Call, FakeFlowActions, FlowOp, FlowPreview, FlowStep, Resume, StepKind};
use super::dialog::{FlowDialog, FlowKind, FlowOutcome, FlowSeed};
use super::test_support::{install_git_flow, view_of};
use super::view::{self, branch_tooltip, grouped_rows};
use crate::refs_tree::{CollapsedFolders, RefRowKind, RefTreeRow};

const BRANCHES: &[&str] = &[
    "main",
    "develop",
    "feature/login",
    "feature/search",
    "hotfix/1.0.1",
    "wip-experiment",
];

fn names() -> Vec<String> {
    BRANCHES.iter().map(|s| s.to_string()).collect()
}

fn key(code: KeyCode) -> KeyEvent {
    KeyEvent::new(code, KeyModifiers::NONE)
}

fn text(rows: &[RefTreeRow]) -> Vec<String> {
    rows.iter()
        .map(|r| match r {
            RefTreeRow::Folder { label, depth, .. } => {
                format!("{}[{label}]", "  ".repeat(*depth as usize))
            }
            RefTreeRow::Leaf {
                display_name,
                depth,
                ..
            } => format!("{}{display_name}", "  ".repeat(*depth as usize)),
        })
        .collect()
}

// -- the grouped sidebar rows ---------------------------------------------

#[test]
fn a_workflow_groups_the_branches_base_first_then_one_folder_per_type() {
    install_git_flow(BRANCHES, "develop");
    let rows = grouped_rows(&names(), RefRowKind::Local, &CollapsedFolders::new())
        .expect("a configured workflow groups the rows");
    let rows = text(&rows);
    // Bases in the workflow's order (main, then develop), with badges;
    // `main` is protected.
    assert_eq!(rows[0], "main  [base, protected]");
    assert_eq!(rows[1], "develop  [base]");
    // Then the types in the workflow's order, each a folder with its count;
    // empty types are not listed.
    assert_eq!(
        &rows[2..],
        [
            "[Feature (2)]",
            "  feature/login",
            "  feature/search",
            "[Hotfix (1)]",
            "  hotfix/1.0.1",
            "[Other (1)]",
            "  wip-experiment",
        ]
    );
}

#[test]
fn collapsing_a_group_hides_its_members_and_keeps_its_count() {
    install_git_flow(BRANCHES, "develop");
    let mut collapsed = CollapsedFolders::new();
    collapsed.insert((RefRowKind::Local, format!("{}feature", view::GROUP_PREFIX)));
    let rows = text(&grouped_rows(&names(), RefRowKind::Local, &collapsed).unwrap());
    assert!(rows.contains(&"[Feature (2)]".to_string()));
    assert!(
        !rows.iter().any(|r| r.contains("feature/login")),
        "{rows:?}"
    );
    assert!(rows.iter().any(|r| r.contains("hotfix/1.0.1")));
}

#[test]
fn the_plain_toggle_and_a_missing_workflow_give_the_plain_tree() {
    // No report at all.
    view::install(None);
    assert!(grouped_rows(&names(), RefRowKind::Local, &CollapsedFolders::new()).is_none());
    install_git_flow(BRANCHES, "develop");
    view::set_plain(true);
    assert!(grouped_rows(&names(), RefRowKind::Local, &CollapsedFolders::new()).is_none());
    // The real tree function takes the grouped rows over the plain ones...
    view::set_plain(false);
    let via_tree = crate::ref_display_rows(
        &names(),
        RefRowKind::Local,
        true,
        &CollapsedFolders::new(),
        &[],
    );
    assert!(text(&via_tree).contains(&"[Feature (2)]".to_string()));
    // ...and Tags are never grouped.
    assert!(grouped_rows(&names(), RefRowKind::Tag, &CollapsedFolders::new()).is_none());
    // The toggle gives the `/`-tree back.
    view::set_plain(true);
    let plain = crate::ref_display_rows(
        &names(),
        RefRowKind::Local,
        true,
        &CollapsedFolders::new(),
        &[],
    );
    assert!(
        text(&plain).contains(&"[feature]".to_string()),
        "{:?}",
        text(&plain)
    );
}

#[test]
fn a_group_folder_stands_for_its_members_when_hiding_or_soloing() {
    install_git_flow(BRANCHES, "develop");
    let path = format!("{}feature", view::GROUP_PREFIX);
    let under = crate::refs_under_folder(&names(), &path);
    assert_eq!(under, ["feature/login", "feature/search"]);
}

#[test]
fn remote_branches_group_by_the_branch_they_name() {
    let view = view_of(
        WorkflowTemplate::GitFlow,
        &["main", "feature/a"],
        "main",
        &[],
    );
    // A remote report entry is keyed `remote/name`.
    let mut report = view.report.clone();
    let mut remote = report.branches[1].clone();
    remote.remote = Some("origin".into());
    report.branches.push(remote);
    super::view::install(Some(Arc::new(super::view::FlowView::new(report))));
    let rows = text(
        &grouped_rows(
            &["origin/feature/a".to_string()],
            RefRowKind::Remote,
            &CollapsedFolders::new(),
        )
        .unwrap(),
    );
    assert_eq!(rows, ["[Feature (1)]", "  origin/feature/a"]);
}

// -- the tooltip -------------------------------------------------------------

#[test]
fn a_branch_tooltip_names_its_type_parent_and_how_far_behind_it_is() {
    let v = view_of(
        WorkflowTemplate::GitFlow,
        BRANCHES,
        "develop",
        &[("feature/login", (2, 3))],
    );
    view::install(Some(v));
    let tip = branch_tooltip("feature/login").unwrap();
    assert_eq!(tip[0], "feature/login");
    assert!(tip.contains(&"Type: Feature".to_string()), "{tip:?}");
    assert!(tip.contains(&"Parent: develop".to_string()), "{tip:?}");
    assert!(
        tip.contains(&"Behind develop by 3 commits".to_string()),
        "{tip:?}"
    );
    assert!(tip.contains(&"Ahead by 2 commits".to_string()), "{tip:?}");
    // A base and an unclassified branch say so.
    assert!(branch_tooltip("main")
        .unwrap()
        .contains(&"Base branch".to_string()));
    assert!(branch_tooltip("wip-experiment")
        .unwrap()
        .contains(&"Not part of the workflow".to_string()));
    // No workflow, no tooltip.
    view::install(None);
    assert!(branch_tooltip("main").is_none());
}

// -- the commands ------------------------------------------------------------

#[test]
fn commands_say_why_they_cannot_run_without_a_workflow_or_a_matching_branch() {
    use super::cmd::FlowCmd;
    view::install(None);
    for cmd in [
        FlowCmd::Menu,
        FlowCmd::StartAny,
        FlowCmd::Finish,
        FlowCmd::Sync,
    ] {
        assert!(cmd.unavailable().is_some(), "{cmd:?}");
    }
    assert!(FlowCmd::Settings.unavailable().is_none());
    // Git Flow with `develop` checked out: Start works, Finish does not
    // (develop is a base), Promote has no environment branch.
    install_git_flow(BRANCHES, "develop");
    assert!(FlowCmd::StartAny.unavailable().is_none());
    assert!(FlowCmd::Start(0).unavailable().is_none());
    assert!(FlowCmd::Start(99).unavailable().is_some());
    assert!(FlowCmd::Finish.unavailable().is_some());
    assert!(FlowCmd::Sync.unavailable().is_none());
    assert!(FlowCmd::Promote.unavailable().is_some());
    install_git_flow(BRANCHES, "feature/login");
    assert!(FlowCmd::Finish.unavailable().is_none());
}

#[test]
fn every_menu_row_has_a_palette_twin() {
    use super::cmd::{menu_rows, FlowCmd};
    use crate::palette::{PaletteCommand, PALETTE_COMMANDS};
    install_git_flow(BRANCHES, "feature/login");
    let rows = menu_rows();
    // One Start row per type, then the fixed rows.
    let types = view::configured().unwrap().topics().len();
    assert!(rows.len() > types);
    let generated = crate::palette::flow_entries();
    for (label, cmd) in rows {
        let found = PALETTE_COMMANDS
            .iter()
            .chain(generated)
            .any(|e| e.command == PaletteCommand::Flow(cmd));
        assert!(found, "menu row {label:?} ({cmd:?}) has no palette command");
    }
    // The generated rows are named per type, in English too.
    let names: Vec<&str> = generated.iter().map(|e| e.name()).collect();
    assert!(names.contains(&"Flow: Start Feature..."), "{names:?}");
    assert!(names.contains(&"Flow: Start Hotfix..."), "{names:?}");
    // The Flow button itself maps to the menu command.
    assert_eq!(
        crate::ToolbarButton::Flow.command(),
        PaletteCommand::Flow(FlowCmd::Menu)
    );
}

#[test]
fn the_toolbar_shows_the_flow_button_only_with_a_workflow() {
    use crate::chrome::{toolbar_render, ToolbarEnabled};
    view::install(None);
    let bar = |w| toolbar_render(Rect::new(0, 0, w, 1), None, ToolbarEnabled::ALL);
    let hit_flow = |bar: &crate::chrome::ToolbarRender| {
        bar.hits
            .iter()
            .any(|(_, t)| *t == crate::HitTarget::Toolbar(crate::ToolbarButton::Flow))
    };
    assert!(!hit_flow(&bar(200)), "no workflow, no button");
    install_git_flow(BRANCHES, "develop");
    let with = bar(200);
    assert!(hit_flow(&with), "a workflow shows it");
    assert!(with.flow_anchor.0 > 0);
}

// -- the dialog ----------------------------------------------------------------

fn seed(kind: FlowKind, head: &str) -> FlowSeed {
    FlowSeed {
        view: view_of(WorkflowTemplate::GitFlow, BRANCHES, head, &[]),
        kind,
        current: Some(head.to_string()),
        selected: None,
        branches: names(),
        tags: vec!["v1.4.2".into(), "v1.3.0".into()],
        issues: vec![
            ("#42".into(), "Fix the Login Bug".into()),
            ("#7".into(), "Other".into()),
        ],
        user: Some("Ada Lovelace".into()),
    }
}

fn preview() -> FlowPreview {
    FlowPreview {
        steps: vec![
            FlowStep::new("Create feature/x from develop", StepKind::Normal),
            FlowStep::new("Push the branch to origin", StepKind::Destructive),
            FlowStep::new("cargo fmt", StepKind::Hook),
        ],
        hooks: vec!["cargo fmt".into()],
        blockers: Vec::new(),
        warnings: vec!["feature/x is 2 commits behind origin/feature/x".into()],
        irreversible: vec!["Pushing feature/x to origin is not taken back by Undo".into()],
        ..Default::default()
    }
}

/// [`preview`] whose hooks the repository committed and nobody trusted yet.
fn untrusted_preview(previous: Option<Vec<gitpoulpi_core::workflow::HookInfo>>) -> FlowPreview {
    FlowPreview {
        trust: gitpoulpi_core::workflow::HooksTrust::Untrusted {
            hash: "ab".repeat(32),
            previous,
        },
        hook_lines: vec![("before_finish".into(), "cargo fmt".into())],
        ..preview()
    }
}

fn fake() -> Arc<FakeFlowActions> {
    FakeFlowActions::new(preview())
}

fn paint(dialog: &mut FlowDialog) -> String {
    let area = Rect::new(0, 0, 110, 26);
    let mut buf = Buffer::empty(area);
    dialog.render(super::flow_rect(area), &mut buf);
    crate::dialog_kit::buffer_text(&buf)
}

fn type_text(dialog: &mut FlowDialog, text: &str) {
    for c in text.chars() {
        dialog.on_key(key(KeyCode::Char(c)));
    }
}

fn wait_poll(dialog: &mut FlowDialog, done: impl Fn(&str) -> bool) -> String {
    let until = Instant::now() + Duration::from_secs(5);
    loop {
        dialog.poll();
        let screen = paint(dialog);
        if done(&screen) {
            return screen;
        }
        assert!(Instant::now() < until, "never got there:\n{screen}");
        std::thread::sleep(Duration::from_millis(5));
    }
}

#[test]
fn the_start_form_builds_the_branch_name_from_the_pattern_and_calls_the_fake() {
    let fake = fake();
    let mut d = FlowDialog::new(seed(FlowKind::Start(Some(0)), "develop"), fake.clone());
    let screen = paint(&mut d);
    for want in [
        "Start Feature",
        "Type",
        "Slug",
        "Start from",
        "develop",
        "Branch",
    ] {
        assert!(screen.contains(want), "{want} missing:\n{screen}");
    }
    // The first text field has the focus: type the slug, the name follows.
    type_text(&mut d, "Add a login page");
    let screen = paint(&mut d);
    assert!(screen.contains("feature/add-a-login-page"), "{screen}");
    // Enter previews, through the fake.
    d.on_key(key(KeyCode::Enter));
    let screen = wait_poll(&mut d, |s| s.contains("Create feature/x from develop"));
    assert!(screen.contains("This will:"), "{screen}");
    let calls = fake.calls();
    let Call::Preview(request) = &calls[0] else {
        panic!("{calls:?}");
    };
    let FlowOp::Start {
        type_id,
        name,
        start_point,
        publish,
        fields,
    } = &request.op
    else {
        panic!("{request:?}");
    };
    assert_eq!(type_id, "feature");
    assert_eq!(name, "feature/add-a-login-page");
    assert_eq!(start_point, "develop");
    assert!(!publish);
    // The fields carry what was typed; the pattern slugifies on render.
    assert_eq!(
        fields.get("slug").map(String::as_str),
        Some("Add a login page")
    );
}

#[test]
fn a_picked_issue_fills_the_number_and_the_slug_and_a_taken_name_is_refused() {
    let mut s = seed(FlowKind::Start(None), "develop");
    // A type whose pattern carries the issue: replace feature's pattern.
    let mut report = s.view.report.clone();
    report.workflow.topics[0].pattern = "feature/{issue}-{slug}".into();
    s.view = Arc::new(super::view::FlowView::new(report));
    s.branches.push("feature/42-fix-the-login-bug".into());
    let mut d = FlowDialog::new(s, fake());
    let screen = paint(&mut d);
    assert!(screen.contains("Issue"), "{screen}");
    // Open the issue dropdown from its field with the keyboard and pick the
    // first row.
    // The first text field has the focus; Shift+Tab goes back to the issue
    // dropdown, and Enter opens it.
    d.on_key(key(KeyCode::BackTab));
    d.on_key(key(KeyCode::Enter));
    let screen = paint(&mut d);
    assert!(screen.contains("#42  Fix the Login Bug"), "{screen}");
    d.on_key(key(KeyCode::Enter)); // picks
    let screen = paint(&mut d);
    assert!(
        screen.contains("A branch named feature/42-fix-the-login-bug already exists"),
        "the name is taken:\n{screen}"
    );
    d.on_key(key(KeyCode::Enter));
    let screen = paint(&mut d);
    // Enter on a field previews, and a refused name keeps the form open.
    assert!(!screen.contains("This will:"), "{screen}");
}

#[test]
fn the_version_field_suggests_the_next_release_from_the_tags() {
    let mut d = FlowDialog::new(seed(FlowKind::Start(Some(2)), "develop"), fake());
    let screen = paint(&mut d);
    // The release type: `release/{version}`, the next minor after v1.4.2.
    assert!(screen.contains("Start Release"), "{screen}");
    assert!(screen.contains("release/1.5.0"), "{screen}");
}

#[test]
fn the_preview_lists_steps_hooks_and_marks_the_destructive_one() {
    let fake = fake();
    let mut d = FlowDialog::new(seed(FlowKind::Finish, "feature/login"), fake.clone());
    let screen = wait_poll(&mut d, |s| s.contains("This will:"));
    assert!(
        screen.contains("1. Create feature/x from develop"),
        "{screen}"
    );
    assert!(
        screen.contains("2. ! Push the branch to origin"),
        "{screen}"
    );
    assert!(screen.contains("3. $ cargo fmt"), "{screen}");
    assert!(
        screen.contains("These hooks run shell commands"),
        "the hooks come with a warning:\n{screen}"
    );
    assert!(screen.contains("[ ] Run the hooks"), "{screen}");
    // The destructive step is painted in the warning colour.
    let area = Rect::new(0, 0, 110, 26);
    let mut buf = Buffer::empty(area);
    d.render(super::flow_rect(area), &mut buf);
    let (x, y) = find(&buf, "2. ! Push");
    assert_eq!(buf[(x, y)].fg, crate::th().status.warning);
    let (x, y) = find(&buf, "1. Create");
    assert_ne!(buf[(x, y)].fg, crate::th().status.warning);
}

#[test]
fn warnings_and_what_undo_will_not_take_back_are_shown_and_highlighted() {
    let fake = fake();
    let mut d = FlowDialog::new(seed(FlowKind::Finish, "feature/login"), fake.clone());
    let screen = wait_poll(&mut d, |s| s.contains("This will:"));
    assert!(screen.contains("2 commits behind"), "{screen}");
    assert!(screen.contains("Undo will not take back:"), "{screen}");
    assert!(
        screen.contains("! Pushing feature/x to origin is not taken back by Undo"),
        "{screen}"
    );
    let area = Rect::new(0, 0, 110, 26);
    let mut buf = Buffer::empty(area);
    d.render(super::flow_rect(area), &mut buf);
    let (x, y) = find(&buf, "! Pushing");
    assert_eq!(buf[(x, y)].fg, crate::th().status.warning);
}

#[test]
fn a_type_that_asks_how_to_finish_gets_a_mode_dropdown_and_sends_the_choice() {
    use gitpoulpi_core::workflow::FinishMode;
    let mut wf = WorkflowTemplate::GitFlow.workflow();
    for t in &mut wf.topics {
        if t.id == "feature" {
            t.finish_mode = Some(FinishMode::Ask);
        }
    }
    let mut s = seed(FlowKind::Finish, "feature/login");
    s.view = Arc::new(super::view::FlowView::new(super::test_support::report(
        wf,
        BRANCHES,
        "feature/login",
        &[],
    )));
    let fake = fake();
    let mut d = FlowDialog::new(s, fake.clone());
    let screen = paint(&mut d);
    assert!(screen.contains("Finish by"), "a form first:\n{screen}");
    assert!(
        screen.contains("Merge locally") || screen.contains("merge locally"),
        "{screen}"
    );
    assert!(fake.calls().is_empty(), "nothing asked before Preview");
    // Open the dropdown, take the second row, preview.
    d.on_key(key(KeyCode::Enter));
    d.on_key(key(KeyCode::Down));
    d.on_key(key(KeyCode::Enter));
    d.on_key(key(KeyCode::Tab));
    d.on_key(key(KeyCode::Enter));
    wait_poll(&mut d, |s| s.contains("This will:"));
    let calls = fake.calls();
    let Call::Preview(request) = &calls[0] else {
        panic!("{calls:?}");
    };
    assert!(
        matches!(
            &request.op,
            FlowOp::Finish {
                mode: Some(FinishMode::MergeRequest),
                ..
            }
        ),
        "{request:?}"
    );
}

fn find(buf: &Buffer, needle: &str) -> (u16, u16) {
    find_from(buf, needle, false)
}

/// The last row (a button row sits below the text that mentions it).
fn find_last(buf: &Buffer, needle: &str) -> (u16, u16) {
    find_from(buf, needle, true)
}

fn find_from(buf: &Buffer, needle: &str, from_bottom: bool) -> (u16, u16) {
    let rows: Vec<u16> = (buf.area.y..buf.area.bottom()).collect();
    let rows: Vec<u16> = if from_bottom {
        rows.into_iter().rev().collect()
    } else {
        rows
    };
    for y in rows {
        let row: String = (buf.area.x..buf.area.right())
            .map(|x| buf[(x, y)].symbol().to_string())
            .collect();
        if let Some(at) = row.find(needle) {
            return (row[..at].chars().count() as u16, y);
        }
    }
    panic!("{needle} is not painted");
}

#[test]
fn confirming_runs_the_steps_with_progress_and_hooks_only_when_ticked() {
    let fake = fake();
    let mut d = FlowDialog::new(seed(FlowKind::Finish, "feature/login"), fake.clone());
    wait_poll(&mut d, |s| s.contains("This will:"));
    // The Finish button has the focus: Enter runs, with the hooks off, so the
    // hook step is not part of the run.
    let _ = d.on_key(key(KeyCode::Enter));
    let screen = wait_poll(&mut d, |s| s.contains("Done."));
    assert!(
        screen.contains("[x] Create feature/x from develop"),
        "{screen}"
    );
    assert!(screen.contains("[x] Push the branch to origin"), "{screen}");
    assert!(!screen.contains("cargo fmt"), "hooks off:\n{screen}");
    let calls = fake.calls();
    let Call::Execute(request) = &calls[1] else {
        panic!("{calls:?}");
    };
    assert!(!request.run_hooks);
    assert!(matches!(&request.op, FlowOp::Finish { branch, .. } if branch == "feature/login"));
    // Close.
    assert_eq!(d.on_key(key(KeyCode::Enter)), FlowOutcome::Close);
}

#[test]
fn ticking_run_hooks_sends_it_and_keeps_the_hook_step() {
    let fake = fake();
    let mut d = FlowDialog::new(seed(FlowKind::Finish, "feature/login"), fake.clone());
    wait_poll(&mut d, |s| s.contains("This will:"));
    // Shift+Tab reaches the hooks checkbox, Space ticks it, Tab comes back to
    // the Finish button.
    d.on_key(key(KeyCode::BackTab));
    d.on_key(key(KeyCode::Char(' ')));
    assert!(paint(&mut d).contains("[x] Run the hooks"));
    d.on_key(key(KeyCode::Tab));
    d.on_key(key(KeyCode::Enter));
    let screen = wait_poll(&mut d, |s| s.contains("Done."));
    assert!(screen.contains("cargo fmt"), "{screen}");
    let Call::Execute(request) = &fake.calls()[1] else {
        panic!();
    };
    assert!(request.run_hooks);
}

#[test]
fn a_conflict_offers_continue_and_abort_and_calls_the_matching_resume() {
    let fake = fake();
    *fake.fail_at.lock().unwrap() = Some((0, "CONFLICT in a.txt".into(), true));
    let mut d = FlowDialog::new(seed(FlowKind::Finish, "feature/login"), fake.clone());
    wait_poll(&mut d, |s| s.contains("This will:"));
    d.on_key(key(KeyCode::Enter));
    let screen = wait_poll(&mut d, |s| s.contains("CONFLICT in a.txt"));
    assert!(screen.contains("[!] Create feature/x"), "{screen}");
    for want in ["Continue", "Abort", "Resolve the conflicts"] {
        assert!(screen.contains(want), "{want}:\n{screen}");
    }
    // Continue resumes; the fake finishes.
    d.on_key(key(KeyCode::Enter));
    wait_poll(&mut d, |s| s.contains("Done."));
    assert!(fake.calls().contains(&Call::Resume(Resume::Continue)));

    // Abort, by the mouse: its button is painted and clickable.
    let fake = fake_with_conflict();
    let mut d = FlowDialog::new(seed(FlowKind::Finish, "feature/login"), fake.clone());
    wait_poll(&mut d, |s| s.contains("This will:"));
    d.on_key(key(KeyCode::Enter));
    let area = Rect::new(0, 0, 110, 26);
    let buf = loop {
        d.poll();
        let mut buf = Buffer::empty(area);
        d.render(super::flow_rect(area), &mut buf);
        if crate::dialog_kit::buffer_text(&buf).contains("Abort") {
            break buf;
        }
        std::thread::sleep(Duration::from_millis(5));
    };
    // The button, not the help line that also says Abort.
    let (x, y) = find_last(&buf, " Abort ");
    d.on_mouse(MouseEvent {
        kind: MouseEventKind::Down(MouseButton::Left),
        column: x + 2,
        row: y,
        modifiers: KeyModifiers::NONE,
    });
    wait_poll(&mut d, |s| s.contains("Aborted") || s.contains("Done"));
    assert!(fake.calls().contains(&Call::Resume(Resume::Abort)));
}

fn fake_with_conflict() -> Arc<FakeFlowActions> {
    let fake = fake();
    *fake.fail_at.lock().unwrap() = Some((0, "CONFLICT in a.txt".into(), true));
    fake
}

#[test]
fn a_slow_preview_and_a_slow_step_show_a_spinner_only_after_150_ms() {
    let fake = Arc::new(FakeFlowActions {
        calls: Default::default(),
        preview: std::sync::Mutex::new(Ok(preview())),
        delay_ms: 400,
        fail_at: Default::default(),
        log: None,
    });
    let mut d = FlowDialog::new(seed(FlowKind::Finish, "feature/login"), fake.clone());
    // Immediately: nothing in flight is announced yet.
    let screen = paint(&mut d);
    assert!(!screen.contains("working out the steps"), "{screen}");
    assert!(d.animating(), "the loop must keep drawing for the spinner");
    std::thread::sleep(Duration::from_millis(200));
    let screen = paint(&mut d);
    assert!(screen.contains("working out the steps"), "{screen}");
}

#[test]
fn a_refused_preview_says_why_and_offers_only_close() {
    let fake = fake();
    *fake.preview.lock().unwrap() = Err("The working tree is not clean".into());
    let mut d = FlowDialog::new(seed(FlowKind::Finish, "feature/login"), fake);
    let screen = wait_poll(&mut d, |s| s.contains("working tree is not clean"));
    let buttons = screen.lines().find(|l| l.contains("Cancel")).unwrap();
    assert!(
        !buttons.contains("Finish"),
        "only Cancel is offered:\n{screen}"
    );
}

#[test]
fn promote_and_set_parent_ask_for_their_target_with_a_dropdown() {
    // GitLab Flow has environment branches.
    let view = view_of(
        WorkflowTemplate::GitlabFlow,
        &["main", "pre-production", "production"],
        "main",
        &[],
    );
    let mut s = seed(FlowKind::Promote, "main");
    s.view = view;
    s.branches = vec!["main".into(), "pre-production".into(), "production".into()];
    let fake = fake();
    let mut d = FlowDialog::new(s, fake.clone());
    let screen = paint(&mut d);
    assert!(screen.contains("Promote into"), "{screen}");
    assert!(screen.contains("pre-production (from main)"), "{screen}");
    // Enter on the (focused) dropdown opens it; the first row is selected.
    d.on_key(key(KeyCode::Enter));
    d.on_key(key(KeyCode::Enter));
    // Tab to the primary button and preview.
    d.on_key(key(KeyCode::Tab));
    d.on_key(key(KeyCode::Enter));
    wait_poll(&mut d, |s| s.contains("This will:"));
    let Call::Preview(request) = &fake.calls()[0] else {
        panic!();
    };
    assert_eq!(
        request.op,
        FlowOp::Promote {
            from: "main".into(),
            to: "pre-production".into()
        }
    );

    let mut d = FlowDialog::new(seed(FlowKind::SetParent, "feature/login"), fake.clone());
    let screen = paint(&mut d);
    assert!(screen.contains("New parent"), "{screen}");
    assert!(
        screen.contains("develop"),
        "the current parent is shown:\n{screen}"
    );
}

#[test]
fn an_open_dropdown_owns_the_pointer_and_esc_closes_it_before_the_dialog() {
    let mut d = FlowDialog::new(seed(FlowKind::Start(Some(0)), "develop"), fake());
    // Focus the type dropdown and open it.
    d.on_key(key(KeyCode::BackTab));
    d.on_key(key(KeyCode::BackTab));
    paint(&mut d);
    for _ in 0..8 {
        if paint(&mut d).contains("Hotfix") {
            break;
        }
        d.on_key(key(KeyCode::Tab));
    }
    // The type combo is the first focus stop: cycle until opening shows the
    // other types.
    let mut found = false;
    for _ in 0..12 {
        d.on_key(key(KeyCode::Enter));
        if paint(&mut d).contains("Hotfix") {
            found = true;
            break;
        }
        d.on_key(key(KeyCode::Esc));
        d.on_key(key(KeyCode::Tab));
    }
    assert!(found, "the type dropdown lists the topic types");
    // A click away closes the popup and nothing else happens (modal).
    assert_eq!(
        d.on_mouse(MouseEvent {
            kind: MouseEventKind::Down(MouseButton::Left),
            column: 1,
            row: 1,
            modifiers: KeyModifiers::NONE,
        }),
        FlowOutcome::Stay
    );
    assert_eq!(d.on_key(key(KeyCode::Esc)), FlowOutcome::Close);
}

#[test]
fn the_flow_dialog_has_a_cross_and_a_typed_slug_is_dirty() {
    use crate::dialog_kit::Dismiss;
    let mut d = FlowDialog::new(seed(FlowKind::Start(Some(0)), "develop"), fake());
    let text = paint(&mut d);
    assert!(text.contains('\u{2715}'), "{text}");
    assert!(!text.contains("Esc"), "{text}");
    assert!(d.close_hit().rect().is_some());
    assert!(!d.is_dirty());
    type_text(&mut d, "login");
    assert!(d.is_dirty(), "a typed slug is work worth keeping");
}

#[test]
fn untrusted_hooks_ask_before_anything_runs_and_trust_and_run_sends_the_hash() {
    let fake = FakeFlowActions::new(untrusted_preview(None));
    let mut d = FlowDialog::new(seed(FlowKind::Finish, "feature/login"), fake.clone());
    let screen = wait_poll(&mut d, |s| s.contains("This will:"));
    // No checkbox: the question comes when the action runs.
    assert!(!screen.contains("Run the hooks"), "{screen}");
    assert!(screen.contains("not trusted yet"), "{screen}");
    d.on_key(key(KeyCode::Enter));
    let screen = paint(&mut d);
    for want in [
        "runs these commands on your machine. Trust them?",
        "$ cargo fmt",
        "1. Trust and run",
        "2. Run without hooks",
    ] {
        assert!(screen.contains(want), "{want}:\n{screen}");
    }
    assert!(
        !fake.calls().iter().any(|c| matches!(c, Call::Execute(_))),
        "nothing ran while the question is open"
    );
    // No default: Enter alone answers nothing.
    d.on_key(key(KeyCode::Enter));
    assert!(!fake
        .calls()
        .iter()
        .any(|c| matches!(c, Call::Execute(_) | Call::TrustHooks(_))));
    // An arrow starts the highlight, Enter on it trusts and runs.
    d.on_key(key(KeyCode::Down));
    d.on_key(key(KeyCode::Enter));
    wait_poll(&mut d, |s| s.contains("Done."));
    let calls = fake.calls();
    assert!(
        calls.contains(&Call::TrustHooks("ab".repeat(32))),
        "{calls:?}"
    );
    let Some(Call::Execute(request)) = calls.iter().find(|c| matches!(c, Call::Execute(_))) else {
        panic!("{calls:?}");
    };
    assert!(request.run_hooks, "trusted: the hooks run");
}

#[test]
fn run_without_hooks_never_asks_the_daemon_to_trust() {
    let fake = FakeFlowActions::new(untrusted_preview(None));
    let mut d = FlowDialog::new(seed(FlowKind::Finish, "feature/login"), fake.clone());
    wait_poll(&mut d, |s| s.contains("This will:"));
    d.on_key(key(KeyCode::Enter));
    d.on_key(key(KeyCode::Char('2')));
    let screen = wait_poll(&mut d, |s| s.contains("Done."));
    assert!(
        !screen.contains("cargo fmt"),
        "the hook step is not run:\n{screen}"
    );
    let calls = fake.calls();
    assert!(
        !calls.iter().any(|c| matches!(c, Call::TrustHooks(_))),
        "{calls:?}"
    );
    let Some(Call::Execute(request)) = calls.iter().find(|c| matches!(c, Call::Execute(_))) else {
        panic!("{calls:?}");
    };
    assert!(!request.run_hooks);
}

#[test]
fn escape_cancels_the_question_and_a_second_escape_leaves_the_dialog() {
    let fake = FakeFlowActions::new(untrusted_preview(None));
    let mut d = FlowDialog::new(seed(FlowKind::Finish, "feature/login"), fake.clone());
    wait_poll(&mut d, |s| s.contains("This will:"));
    d.on_key(key(KeyCode::Enter));
    assert!(paint(&mut d).contains("Trust them?"));
    assert_eq!(d.on_key(key(KeyCode::Esc)), FlowOutcome::Stay);
    let screen = paint(&mut d);
    assert!(!screen.contains("Trust them?"), "{screen}");
    assert!(
        screen.contains("This will:"),
        "back on the preview:\n{screen}"
    );
    assert!(!fake
        .calls()
        .iter()
        .any(|c| matches!(c, Call::Execute(_) | Call::TrustHooks(_))));
    assert_eq!(d.on_key(key(KeyCode::Esc)), FlowOutcome::Close);
}

#[test]
fn changed_hooks_say_so_and_show_the_old_list_and_the_buttons_answer_like_the_keys() {
    let old = vec![gitpoulpi_core::workflow::HookInfo {
        phase: "before_finish".into(),
        command: "make old".into(),
        personal: false,
    }];
    let fake = FakeFlowActions::new(untrusted_preview(Some(old)));
    let mut d = FlowDialog::new(seed(FlowKind::Finish, "feature/login"), fake.clone());
    wait_poll(&mut d, |s| s.contains("This will:"));
    d.on_key(key(KeyCode::Enter));
    let screen = paint(&mut d);
    assert!(
        screen.contains("The hooks changed since you trusted them"),
        "{screen}"
    );
    assert!(screen.contains("$ make old"), "{screen}");
    // The Cancel button is the mouse twin of Esc.
    let area = Rect::new(0, 0, 110, 26);
    let mut buf = Buffer::empty(area);
    d.render(super::flow_rect(area), &mut buf);
    let (x, y) = find(&buf, "Cancel");
    d.on_mouse(MouseEvent {
        kind: MouseEventKind::Down(MouseButton::Left),
        column: x + 2,
        row: y,
        modifiers: KeyModifiers::NONE,
    });
    assert!(!paint(&mut d).contains("Trust them?"));
    // And a click on answer 2 runs without hooks.
    d.on_key(key(KeyCode::Enter));
    let mut buf = Buffer::empty(area);
    d.render(super::flow_rect(area), &mut buf);
    let (x, y) = find(&buf, "2. Run without hooks");
    d.on_mouse(MouseEvent {
        kind: MouseEventKind::Down(MouseButton::Left),
        column: x + 3,
        row: y,
        modifiers: KeyModifiers::NONE,
    });
    wait_poll(&mut d, |s| s.contains("Done."));
    assert!(fake
        .calls()
        .iter()
        .any(|c| matches!(c, Call::Execute(r) if !r.run_hooks)));
}

#[test]
fn trusted_and_personal_hooks_keep_the_checkbox_and_never_ask() {
    for trust in [
        gitpoulpi_core::workflow::HooksTrust::Trusted,
        gitpoulpi_core::workflow::HooksTrust::Personal,
    ] {
        let fake = FakeFlowActions::new(FlowPreview {
            trust,
            hook_lines: vec![("before_finish".into(), "cargo fmt".into())],
            ..preview()
        });
        let mut d = FlowDialog::new(seed(FlowKind::Finish, "feature/login"), fake.clone());
        let screen = wait_poll(&mut d, |s| s.contains("This will:"));
        assert!(screen.contains("[ ] Run the hooks"), "{screen}");
        d.on_key(key(KeyCode::Enter));
        wait_poll(&mut d, |s| s.contains("Done."));
        assert!(!fake
            .calls()
            .iter()
            .any(|c| matches!(c, Call::TrustHooks(_))));
    }
}
