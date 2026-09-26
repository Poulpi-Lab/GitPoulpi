use super::*;
use crate::hosting_task::ClientFactory;
use gitpoulpi_hosting::{
    HostedLabel, HostedMilestone, HostedProject, HostedSshKey, HostedUser, HostingClient,
    Integration, IntegrationKind, ItemMeta, MetaKind, NewPullRequest,
};
use ratatui::buffer::Buffer;
use std::sync::Arc;

struct Fake {
    integration: Integration,
    fail: bool,
}

#[async_trait::async_trait]
impl HostingClient for Fake {
    fn integration(&self) -> &Integration {
        &self.integration
    }
    async fn list_projects(&self) -> gitpoulpi_hosting::Result<Vec<HostedProject>> {
        Ok(Vec::new())
    }
    async fn list_ssh_keys(&self) -> gitpoulpi_hosting::Result<Vec<HostedSshKey>> {
        Ok(Vec::new())
    }
    async fn upload_ssh_key(&self, _: &str, _: &str) -> gitpoulpi_hosting::Result<HostedSshKey> {
        Err(gitpoulpi_hosting::Error::Unsupported("upload".into()))
    }
    async fn create_pull_request(
        &self,
        _: &NewPullRequest,
    ) -> gitpoulpi_hosting::Result<gitpoulpi_hosting::CreatedPullRequest> {
        Err(gitpoulpi_hosting::Error::Unsupported("create".into()))
    }
    async fn project_members(
        &self,
        _: &str,
        _: &str,
    ) -> gitpoulpi_hosting::Result<Vec<HostedUser>> {
        if self.fail {
            return Err(gitpoulpi_hosting::Error::Http("host unreachable".into()));
        }
        let person = |login: &str, name: &str| HostedUser {
            login: login.into(),
            name: name.into(),
            id: "1".into(),
            web_url: None,
        };
        Ok(vec![
            person("ann", "Ann Archer"),
            person("bob", "Bob Baker"),
            person("cyd", "Cyd Cole"),
        ])
    }
    async fn project_labels(&self, _: &str) -> gitpoulpi_hosting::Result<Vec<HostedLabel>> {
        Ok(vec![
            HostedLabel {
                name: "bug".into(),
                color: "d73a4a".into(),
                description: "Broken".into(),
            },
            HostedLabel {
                name: "ui".into(),
                color: "0075ca".into(),
                description: String::new(),
            },
        ])
    }
    async fn project_milestones(&self, _: &str) -> gitpoulpi_hosting::Result<Vec<HostedMilestone>> {
        Ok(vec![
            HostedMilestone {
                id: 7,
                title: "v1.0".into(),
                due_on: Some("2026-12-01".into()),
                state: "open".into(),
            },
            HostedMilestone {
                id: 8,
                title: "v1.1".into(),
                due_on: None,
                state: "open".into(),
            },
        ])
    }
    async fn item_meta(&self, _: &str, _: u64, _: MetaKind) -> gitpoulpi_hosting::Result<ItemMeta> {
        Ok(ItemMeta {
            assignees: vec!["bob".into()],
            labels: vec!["bug".into()],
            milestone: Some(HostedMilestone {
                id: 8,
                title: "v1.1".into(),
                ..Default::default()
            }),
            ..Default::default()
        })
    }
}

struct FakeFactory {
    fail: bool,
}

impl ClientFactory for FakeFactory {
    fn client(
        &self,
        integration: &Integration,
        _: &[Integration],
    ) -> gitpoulpi_hosting::Result<Box<dyn HostingClient>> {
        Ok(Box::new(Fake {
            integration: integration.clone(),
            fail: self.fail,
        }))
    }
}

fn target(number: Option<u64>) -> MetaTarget {
    MetaTarget {
        integration: Integration::new("gh", IntegrationKind::Github, "https://github.com"),
        all: Vec::new(),
        project: "me/p".into(),
        kind: MetaKind::Issue,
        number,
        caption: "#7 Fix the thing".into(),
    }
}

fn picker(field: MetaField, number: Option<u64>, fail: bool) -> MetaPicker {
    let mut p = MetaPicker::open(
        Arc::new(FakeFactory { fail }),
        target(number),
        field,
        &MetaSelection::default(),
    );
    let started = Instant::now();
    while p.busy() {
        assert!(started.elapsed() < Duration::from_secs(10), "never loaded");
        p.poll();
        std::thread::sleep(Duration::from_millis(5));
    }
    p
}

fn key(code: KeyCode) -> KeyEvent {
    KeyEvent::new(code, KeyModifiers::NONE)
}

fn typed(p: &mut MetaPicker, text: &str) {
    for c in text.chars() {
        p.on_key(key(KeyCode::Char(c)));
    }
}

fn draw(p: &mut MetaPicker) -> String {
    let mut buf = Buffer::empty(Rect::new(0, 0, 100, 30));
    p.render(buf.area, &mut buf, None);
    crate::dialog_kit::buffer_text(&buf)
}

fn click_at(p: &mut MetaPicker, rect: Rect) -> PickerOutcome {
    p.on_mouse(MouseEvent {
        kind: MouseEventKind::Down(MouseButton::Left),
        column: rect.x + 1,
        row: rect.y,
        modifiers: KeyModifiers::NONE,
    })
}

fn hit_rect(p: &MetaPicker, hit: Hit) -> Rect {
    p.hits
        .iter()
        .find(|(_, h)| *h == hit)
        .map(|(r, _)| *r)
        .unwrap_or_else(|| panic!("{hit:?} not painted: {:?}", p.hits))
}

#[test]
fn people_load_with_badges_and_the_items_current_assignees_are_checked() {
    let mut p = picker(MetaField::Assignees, Some(7), false);
    let screen = draw(&mut p);
    for want in [
        "Edit assignees",
        "#7 Fix the thing",
        "ann",
        "Bob Baker",
        "cyd",
    ] {
        assert!(screen.contains(want), "{want}:\n{screen}");
    }
    // Initials badge for a person (the avatar fallback), checkbox state of
    // the item's current assignee.
    assert!(screen.contains("AA"), "Ann Archer's initials:\n{screen}");
    assert_eq!(p.checked(), ["bob"]);
    assert!(screen.contains("[x]") && screen.contains("[ ]"), "{screen}");
}

#[test]
fn typing_filters_space_toggles_and_enter_applies_in_the_order_checked() {
    let mut p = picker(MetaField::Assignees, None, false);
    typed(&mut p, "cy");
    assert_eq!(p.shown.len(), 1, "the filter narrows the list");
    assert!(draw(&mut p).contains("cyd"));
    p.on_key(key(KeyCode::Char(' ')));
    assert_eq!(p.checked(), ["cyd"]);
    // Backspace twice: everything is back; check Ann too.
    p.on_key(key(KeyCode::Backspace));
    p.on_key(key(KeyCode::Backspace));
    assert_eq!(p.shown.len(), 3);
    p.on_key(key(KeyCode::Char(' ')));
    assert_eq!(p.checked(), ["cyd", "ann"]);
    // The host's own order on the way out, not the click order.
    assert_eq!(
        p.on_key(key(KeyCode::Enter)),
        PickerOutcome::Apply(Applied::List(
            MetaField::Assignees,
            vec!["ann".into(), "cyd".into()]
        ))
    );
}

#[test]
fn a_click_on_a_row_toggles_it_and_the_buttons_click() {
    let mut p = picker(MetaField::Labels, Some(7), false);
    draw(&mut p);
    assert_eq!(p.checked(), ["bug"]);
    // `ui` is the second shown row.
    let row = hit_rect(&p, Hit::Row(1));
    assert_eq!(click_at(&mut p, row), PickerOutcome::Stay);
    assert_eq!(p.checked(), ["bug", "ui"]);
    draw(&mut p);
    let row = hit_rect(&p, Hit::Row(0));
    click_at(&mut p, row);
    assert_eq!(p.checked(), ["ui"], "a second click unchecks");
    draw(&mut p);
    let apply = hit_rect(&p, Hit::Apply);
    assert_eq!(
        click_at(&mut p, apply),
        PickerOutcome::Apply(Applied::List(MetaField::Labels, vec!["ui".into()]))
    );
    draw(&mut p);
    let cancel = hit_rect(&p, Hit::Cancel);
    assert_eq!(click_at(&mut p, cancel), PickerOutcome::Cancel);
}

#[test]
fn a_swatch_paints_for_a_label_with_a_colour() {
    let mut p = picker(MetaField::Labels, None, false);
    let mut buf = Buffer::empty(Rect::new(0, 0, 100, 30));
    p.render(buf.area, &mut buf, None);
    let (r, _) = *p.hits.iter().find(|(_, h)| *h == Hit::Row(0)).unwrap();
    // `[x] ` is 4 cells: the 2-cell swatch follows, on the label's colour.
    let swatch = &buf[(r.x + 4, r.y)];
    assert_eq!(swatch.bg, ratatui::style::Color::Rgb(0xd7, 0x3a, 0x4a));
}

#[test]
fn escape_cancels_and_tab_reaches_the_buttons() {
    let mut p = picker(MetaField::Assignees, None, false);
    assert_eq!(p.on_key(key(KeyCode::Esc)), PickerOutcome::Cancel);
    p.on_key(key(KeyCode::Tab));
    p.on_key(key(KeyCode::Tab));
    assert_eq!(p.focus, Focus::Cancel);
    assert_eq!(p.on_key(key(KeyCode::Enter)), PickerOutcome::Cancel);
}

#[test]
fn the_milestone_is_a_combobox_with_the_current_value_and_no_milestone() {
    let mut p = picker(MetaField::Milestone, Some(7), false);
    let screen = draw(&mut p);
    assert!(screen.contains("Set milestone"), "{screen}");
    assert!(p.combo.is_open(), "the list is open on load");
    assert!(
        screen.contains("No milestone") && screen.contains("v1.0"),
        "{screen}"
    );
    assert_eq!(p.combo.selected(), Some("8"), "the item's milestone");
    // Down to `v1.0` (row 1), Enter picks it and the focus moves to Apply.
    p.on_key(key(KeyCode::Up));
    p.on_key(key(KeyCode::Enter));
    assert!(!p.combo.is_open());
    assert_eq!(p.focus, Focus::Apply);
    assert_eq!(
        p.on_key(key(KeyCode::Enter)),
        PickerOutcome::Apply(Applied::Milestone(Some(MilestoneRef {
            id: 7,
            title: "v1.0".into()
        })))
    );
}

#[test]
fn choosing_no_milestone_clears_it() {
    let mut p = picker(MetaField::Milestone, Some(7), false);
    draw(&mut p);
    for _ in 0..2 {
        p.on_key(key(KeyCode::Up));
    }
    p.on_key(key(KeyCode::Enter));
    assert_eq!(
        p.on_key(key(KeyCode::Enter)),
        PickerOutcome::Apply(Applied::Milestone(None))
    );
}

#[test]
fn the_spinner_row_shows_only_after_150_ms() {
    let mut p = MetaPicker::open(
        Arc::new(FakeFactory { fail: false }),
        target(None),
        MetaField::Assignees,
        &MetaSelection::default(),
    );
    // Drop the answer so the picker stays in flight.
    p.rx = None;
    p.status = Status::Loading;
    p.since = Instant::now();
    assert!(!draw(&mut p).contains("loading"), "not yet");
    p.since = Instant::now() - Duration::from_millis(300);
    assert!(
        draw(&mut p).contains("loading..."),
        "after the grace period"
    );
}

#[test]
fn a_failed_load_says_why_and_refuses_to_apply() {
    let mut p = picker(MetaField::Assignees, None, true);
    let screen = draw(&mut p);
    assert!(screen.contains("host unreachable"), "{screen}");
    assert_eq!(p.on_key(key(KeyCode::Enter)), PickerOutcome::Stay);
}

#[test]
fn applying_before_the_host_answered_is_refused() {
    let mut p = MetaPicker::open(
        Arc::new(FakeFactory { fail: false }),
        target(None),
        MetaField::Labels,
        &MetaSelection::default(),
    );
    p.rx = None;
    p.status = Status::Loading;
    assert_eq!(p.on_key(key(KeyCode::Enter)), PickerOutcome::Stay);
    assert!(p.message.as_deref().is_some_and(|m| m.contains("loading")));
}

#[test]
fn a_create_form_starts_from_what_it_chose_and_forgets_it_for_another_project() {
    let mut fields = MetaFields::default();
    fields.set_scope("gh|me/p".into());
    fields
        .selection
        .apply(Applied::List(MetaField::Labels, vec!["bug".into()]));
    fields.set_scope("gh|me/p".into());
    assert_eq!(fields.selection.labels, ["bug"], "same scope keeps it");
    fields.open(
        Arc::new(FakeFactory { fail: false }),
        target(None),
        MetaField::Labels,
    );
    assert!(fields.is_open());
    let started = Instant::now();
    while fields.busy() {
        assert!(started.elapsed() < Duration::from_secs(10));
        fields.poll();
        std::thread::sleep(Duration::from_millis(5));
    }
    let outcome = fields.picker.as_ref().unwrap().checked().to_vec();
    assert_eq!(outcome, ["bug"], "the picker opens on the form's choice");
    fields.on_key(key(KeyCode::Enter));
    assert!(!fields.is_open());
    assert_eq!(fields.selection.labels, ["bug"]);
    fields.set_scope("gh|other/q".into());
    assert!(
        fields.selection.labels.is_empty(),
        "another project, a fresh choice"
    );
}

#[test]
fn a_selection_summarizes_for_the_form_chips() {
    let mut s = MetaSelection::default();
    assert_eq!(s.summary(MetaField::Assignees), "none");
    s.apply(Applied::List(MetaField::Assignees, vec!["ann".into()]));
    assert_eq!(s.summary(MetaField::Assignees), "ann");
    s.apply(Applied::List(
        MetaField::Assignees,
        vec!["ann".into(), "bob".into(), "cyd".into()],
    ));
    assert_eq!(s.summary(MetaField::Assignees), "ann +2");
    s.apply(Applied::Milestone(Some(MilestoneRef {
        id: 1,
        title: "v1".into(),
    })));
    assert_eq!(s.summary(MetaField::Milestone), "v1");
}

#[test]
fn only_github_gitlab_gitea_linear_and_youtrack_take_metadata() {
    for kind in [
        IntegrationKind::Github,
        IntegrationKind::GithubEnterprise,
        IntegrationKind::Gitlab,
        IntegrationKind::GitlabSelfManaged,
        IntegrationKind::Gitea,
        IntegrationKind::Forgejo,
        IntegrationKind::Linear,
        IntegrationKind::Youtrack,
        IntegrationKind::Redmine,
    ] {
        assert!(supports(kind), "{kind:?}");
    }
    for kind in [
        IntegrationKind::Bitbucket,
        IntegrationKind::JiraCloud,
        IntegrationKind::Trello,
        IntegrationKind::Gerrit,
    ] {
        assert!(!supports(kind), "{kind:?}");
    }
}

#[test]
fn redmine_has_no_labels_so_the_label_picker_is_hidden() {
    for field in [MetaField::Assignees, MetaField::Milestone] {
        assert!(supports_field(IntegrationKind::Redmine, field), "{field:?}");
    }
    assert!(!supports_field(IntegrationKind::Redmine, MetaField::Labels));
    // Every other provider that takes metadata keeps the label picker.
    for kind in [
        IntegrationKind::Github,
        IntegrationKind::Gitlab,
        IntegrationKind::Gitea,
        IntegrationKind::Linear,
        IntegrationKind::Youtrack,
    ] {
        assert!(supports_field(kind, MetaField::Labels), "{kind:?}");
    }
}

fn click_xy(p: &mut MetaPicker, x: u16, y: u16) -> PickerOutcome {
    p.on_mouse(MouseEvent {
        kind: MouseEventKind::Down(MouseButton::Left),
        column: x,
        row: y,
        modifiers: KeyModifiers::NONE,
    })
}

#[test]
fn the_cross_closes_and_an_outside_click_closes_unless_the_selection_changed() {
    let mut p = picker(MetaField::Labels, Some(7), false);
    let text = draw(&mut p);
    assert!(text.contains('\u{2715}'), "{text}");
    assert!(!text.contains("Esc"), "no Esc hint\n{text}");
    // Untouched: a click outside closes.
    assert_eq!(click_xy(&mut p, 0, 0), PickerOutcome::Cancel);
    // Touched: it is swallowed (the cross flashes), the cross still closes.
    let row = hit_rect(&p, Hit::Row(1));
    click_at(&mut p, row);
    draw(&mut p);
    assert_eq!(click_xy(&mut p, 0, 0), PickerOutcome::Stay);
    // A click inside the frame on no control does nothing.
    let inside = p.frame;
    assert_eq!(
        click_xy(&mut p, inside.x + 1, inside.y + 1),
        PickerOutcome::Stay
    );
    let cross = p.close.rect().expect("a cross");
    assert_eq!(
        click_xy(&mut p, cross.x + 1, cross.y),
        PickerOutcome::Cancel
    );
}

/// A value row is the arrow and lights as a whole under the pointer; its
/// checkbox mark keeps the hand.
#[test]
fn a_value_row_is_the_arrow_lit_as_a_whole_and_its_checkbox_the_hand() {
    use crate::pointer_map::{self, Kind};
    let mut p = picker(MetaField::Labels, Some(7), false);
    pointer_map::begin_frame();
    draw(&mut p);
    let row = hit_rect(&p, Hit::Row(1));
    assert_eq!(pointer_map::at(row.x + 8, row.y), Some(Kind::Plain));
    assert_eq!(pointer_map::at(row.x + 1, row.y), Some(Kind::Hand));
    p.hover = Some(Hit::Row(1));
    let mut buf = Buffer::empty(Rect::new(0, 0, 100, 30));
    p.render(buf.area, &mut buf, None);
    for x in [row.x, row.x + 1, row.x + 10, row.right() - 1] {
        assert_eq!(buf[(x, row.y)].bg, crate::th().ui.hover, "col {x}");
    }
    let other = hit_rect(&p, Hit::Row(0));
    assert_ne!(buf[(other.x + 8, other.y)].bg, crate::th().ui.hover);
}
