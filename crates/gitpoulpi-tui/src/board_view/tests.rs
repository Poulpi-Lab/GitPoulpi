//! The board over a fake provider: the cards come from memory, a move is
//! really sent to the fake (and can be made to fail or to be slow), and
//! everything else - the layout, the keys, the mouse, the filters - is the
//! real code. The paint is read back from a `Buffer`.

use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers, MouseButton, MouseEvent, MouseEventKind};
use gitpoulpi_hosting::board::{BoardColumn, BoardSource, BoardSpec};
use gitpoulpi_hosting::{
    HostedProject, HostedSshKey, HostingClient, Integration, IntegrationKind, IssueFilter,
    IssueState, IssueSummary, NewPullRequest, StateFilter,
};
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;

use super::draw::{layout, Hit};
use super::*;
use crate::board_cmd::BoardCmd;
use crate::hosting_task::ClientFactory;

const AREA: Rect = Rect {
    x: 0,
    y: 0,
    width: 130,
    height: 36,
};

#[derive(Default)]
struct Shared {
    issues: Vec<IssueSummary>,
    moves: Vec<(u64, String)>,
}

struct Fake {
    integration: Integration,
    source: BoardSource,
    shared: Arc<Mutex<Shared>>,
    fail: Option<&'static str>,
    delay: Duration,
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
    fn board_source(&self) -> BoardSource {
        self.source
    }
    async fn list_issues(
        &self,
        _: &str,
        filter: &IssueFilter,
    ) -> gitpoulpi_hosting::Result<Vec<IssueSummary>> {
        let all = self.shared.lock().unwrap().issues.clone();
        Ok(all
            .into_iter()
            .filter(|i| match filter.state {
                StateFilter::Open => i.state == IssueState::Open,
                StateFilter::Closed => i.state == IssueState::Closed,
                StateFilter::All => true,
            })
            .collect())
    }
    async fn issue_statuses(&self, _: &str) -> gitpoulpi_hosting::Result<Vec<String>> {
        Ok(vec!["todo".into(), "doing".into(), "done".into()])
    }
    async fn move_issue(
        &self,
        _: &str,
        issue: &IssueSummary,
        spec: &BoardSpec,
        to: &BoardColumn,
    ) -> gitpoulpi_hosting::Result<()> {
        std::thread::sleep(self.delay);
        if let Some(why) = self.fail {
            return Err(gitpoulpi_hosting::Error::Local(why.into()));
        }
        let plan = gitpoulpi_hosting::board::plan_move(spec, issue, to)?;
        let mut shared = self.shared.lock().unwrap();
        shared.moves.push((issue.number, to.name.clone()));
        if let Some(i) = shared.issues.iter_mut().find(|i| i.number == issue.number) {
            gitpoulpi_hosting::board::apply_plan(i, &plan);
        }
        Ok(())
    }
}

struct Factory {
    shared: Arc<Mutex<Shared>>,
    source: BoardSource,
    fail: Option<&'static str>,
    delay: Duration,
}

impl ClientFactory for Factory {
    fn client(
        &self,
        integration: &Integration,
        _: &[Integration],
    ) -> gitpoulpi_hosting::Result<Box<dyn HostingClient>> {
        Ok(Box::new(Fake {
            integration: integration.clone(),
            source: self.source,
            shared: self.shared.clone(),
            fail: self.fail,
            delay: self.delay,
        }))
    }
}

fn issue(n: u64, status: &str) -> IssueSummary {
    IssueSummary {
        number: n,
        id: n.to_string(),
        reference: format!("#{n}"),
        iid_label: format!("#{n}"),
        title: format!("Card number {n} has a rather long title that wraps"),
        status_label: Some(status.into()),
        state: if status == "done" {
            IssueState::Closed
        } else {
            IssueState::Open
        },
        assignees: vec!["Morgane Lefebvre".into()],
        labels: vec!["bug".into()],
        milestone: Some("v0.1".into()),
        web_url: format!("https://example.test/issues/{n}"),
        ..IssueSummary::default()
    }
}

struct Rig {
    view: BoardView,
    shared: Arc<Mutex<Shared>>,
    /// The repository root: the saved column labels live under it.
    dir: tempfile::TempDir,
}

fn rig_with(source: BoardSource, fail: Option<&'static str>, delay: Duration) -> Rig {
    let shared = Arc::new(Mutex::new(Shared {
        issues: vec![
            issue(1, "todo"),
            issue(2, "todo"),
            issue(3, "doing"),
            issue(4, "done"),
        ],
        moves: Vec::new(),
    }));
    let factory = Arc::new(Factory {
        shared: shared.clone(),
        source,
        fail,
        delay,
    });
    let dir = tempfile::tempdir().unwrap();
    let integration = Integration::new("corp", IntegrationKind::Gitlab, "https://gitlab.example");
    let target = BoardTarget {
        seed: IssueSeed {
            integration: integration.clone(),
            provider: "corp".into(),
            project: "g/p".into(),
        },
        all: vec![integration],
        query: None,
        repo_root: dir.path().to_path_buf(),
        labels: Vec::new(),
    };
    let mut view = BoardView::open(target, factory);
    wait_for(&mut view, |v| v.loaded);
    Rig { view, shared, dir }
}

fn rig() -> Rig {
    rig_with(BoardSource::LocalStatus, None, Duration::ZERO)
}

fn wait_for(view: &mut BoardView, done: impl Fn(&BoardView) -> bool) {
    let until = Instant::now() + Duration::from_secs(5);
    while !done(view) {
        view.poll();
        assert!(Instant::now() < until, "the board never got there");
        std::thread::sleep(Duration::from_millis(5));
    }
}

fn paint(view: &mut BoardView) -> String {
    let mut buf = Buffer::empty(AREA);
    view.render(AREA, &mut buf);
    crate::dialog_kit::buffer_text(&buf)
}

fn key(code: KeyCode) -> KeyEvent {
    KeyEvent::new(code, KeyModifiers::NONE)
}

fn shift(code: KeyCode) -> KeyEvent {
    KeyEvent::new(code, KeyModifiers::SHIFT)
}

fn mouse(kind: MouseEventKind, col: u16, row: u16) -> MouseEvent {
    MouseEvent {
        kind,
        column: col,
        row,
        modifiers: KeyModifiers::NONE,
    }
}

fn hit_rect(view: &BoardView, hit: Hit) -> Rect {
    view.hits
        .iter()
        .find(|(_, h)| *h == hit)
        .unwrap_or_else(|| panic!("no {hit:?} was painted"))
        .0
}

fn column_numbers(view: &BoardView, col: usize) -> Vec<u64> {
    view.cards[col]
        .iter()
        .map(|&i| view.issues[i].number)
        .collect()
}

#[test]
fn the_columns_come_from_the_declared_statuses_with_their_counts_and_cards() {
    let mut r = rig();
    let text = paint(&mut r.view);
    for want in [
        "todo", "doing", "done", "#1", "#3", "#4", "v0.1", "bug", "ML",
    ] {
        assert!(text.contains(want), "{want} is not painted:\n{text}");
    }
    assert_eq!(column_numbers(&r.view, 0), [1, 2]);
    assert_eq!(column_numbers(&r.view, 1), [3]);
    assert_eq!(column_numbers(&r.view, 2), [4]);
    // The title wraps on two lines at most, never one long line.
    assert!(text.contains("Card number 1 has a"), "{text}");
}

#[test]
fn an_empty_column_is_still_shown() {
    let mut r = rig();
    r.shared.lock().unwrap().issues.retain(|i| i.number != 3);
    r.view.refresh();
    wait_for(&mut r.view, |v| v.load_rx.is_none());
    let text = paint(&mut r.view);
    assert!(text.contains("doing"), "{text}");
    assert!(text.contains("No cards"), "{text}");
}

#[test]
fn shift_right_moves_the_card_at_once_then_the_provider_confirms() {
    let mut r = rig_with(BoardSource::LocalStatus, None, Duration::from_millis(200));
    paint(&mut r.view);
    assert_eq!(r.view.sel_number, Some(1));
    r.view.on_key(shift(KeyCode::Right));
    // Optimistic: on the board before the provider answered.
    assert_eq!(column_numbers(&r.view, 1), [1, 3]);
    assert!(r.shared.lock().unwrap().moves.is_empty());
    assert_eq!(r.view.sel_col, 1, "the selection follows the card");
    // After 150 ms the card shows it is in flight.
    std::thread::sleep(Duration::from_millis(160));
    let text = paint(&mut r.view);
    assert!(text.contains("moving #1"), "no in-flight line:\n{text}");
    wait_for(&mut r.view, |v| v.moves.is_empty());
    assert_eq!(r.shared.lock().unwrap().moves, [(1, "doing".to_string())]);
    assert_eq!(column_numbers(&r.view, 1), [1, 3]);
    assert!(r.view.message.as_deref().unwrap_or("").contains("moved #1"));
}

#[test]
fn a_refused_move_snaps_the_card_back_and_says_why() {
    let mut r = rig_with(
        BoardSource::LocalStatus,
        Some("the folder is read-only"),
        Duration::ZERO,
    );
    r.view.on_key(shift(KeyCode::Right));
    assert_eq!(column_numbers(&r.view, 1), [1, 3], "optimistic first");
    wait_for(&mut r.view, |v| v.moves.is_empty());
    assert_eq!(column_numbers(&r.view, 0), [1, 2], "back where it was");
    let text = paint(&mut r.view);
    assert!(text.contains("the folder is read-only"), "{text}");
    assert!(r.shared.lock().unwrap().moves.is_empty());
}

#[test]
fn dragging_a_card_onto_another_column_moves_it_and_highlights_the_drop() {
    let mut r = rig();
    paint(&mut r.view);
    let card = hit_rect(&r.view, Hit::Card(0, 0));
    let target = hit_rect(&r.view, Hit::Header(2));
    r.view.on_mouse(mouse(
        MouseEventKind::Down(MouseButton::Left),
        card.x + 3,
        card.y + 1,
    ));
    assert!(r.view.drag.is_some());
    r.view.on_mouse(mouse(
        MouseEventKind::Drag(MouseButton::Left),
        target.x + 4,
        target.y + 6,
    ));
    let drag = r.view.drag.expect("dragging");
    assert!(drag.moved);
    assert_eq!(drag.over, Some(2), "the column under the pointer");
    // The drop target is lit: its header is painted in the accent color.
    let mut buf = Buffer::empty(AREA);
    r.view.render(AREA, &mut buf);
    let header = &buf[(target.x + 1, target.y)];
    assert_eq!(header.bg, crate::th().ui.accent, "drop highlight");
    r.view.on_mouse(mouse(
        MouseEventKind::Up(MouseButton::Left),
        target.x + 4,
        target.y + 6,
    ));
    assert!(r.view.drag.is_none());
    assert_eq!(column_numbers(&r.view, 2), [1, 4]);
    wait_for(&mut r.view, |v| v.moves.is_empty());
    assert_eq!(r.shared.lock().unwrap().moves, [(1, "done".to_string())]);
}

#[test]
fn a_click_without_a_drag_only_selects_and_a_double_click_opens() {
    let mut r = rig();
    paint(&mut r.view);
    let card = hit_rect(&r.view, Hit::Card(0, 1));
    let down = mouse(MouseEventKind::Down(MouseButton::Left), card.x + 2, card.y);
    let up = mouse(MouseEventKind::Up(MouseButton::Left), card.x + 2, card.y);
    assert_eq!(r.view.on_mouse(down), BoardEffect::None);
    assert_eq!(r.view.on_mouse(up), BoardEffect::None);
    assert_eq!(r.view.sel_number, Some(2));
    assert!(r.shared.lock().unwrap().moves.is_empty());
    assert_eq!(
        r.view.on_mouse(down),
        BoardEffect::OpenUrl("https://example.test/issues/2".into())
    );
}

#[test]
fn arrows_move_the_selection_and_enter_opens_the_issue() {
    let mut r = rig();
    paint(&mut r.view);
    r.view.on_key(key(KeyCode::Down));
    assert_eq!(r.view.sel_number, Some(2));
    r.view.on_key(key(KeyCode::Right));
    assert_eq!(
        r.view.sel_number,
        Some(3),
        "the nearest card of the next column"
    );
    assert_eq!(
        r.view.on_key(key(KeyCode::Enter)),
        BoardEffect::OpenUrl("https://example.test/issues/3".into())
    );
    assert_eq!(r.view.on_key(key(KeyCode::Esc)), BoardEffect::Close);
}

#[test]
fn the_move_card_to_dropdown_lists_the_columns_and_moves_the_card() {
    let mut r = rig();
    paint(&mut r.view);
    r.view.on_key(key(KeyCode::Char('m')));
    paint(&mut r.view);
    let combo = r.view.move_combo.as_ref().expect("the dropdown is open");
    assert!(combo.is_open());
    let labels: Vec<&str> = combo.options.iter().map(|o| o.label.as_str()).collect();
    assert_eq!(labels, ["todo", "doing", "done"]);
    r.view.on_key(key(KeyCode::Down));
    r.view.on_key(key(KeyCode::Enter));
    assert!(r.view.move_combo.is_none());
    assert_eq!(column_numbers(&r.view, 1), [1, 3]);
}

#[test]
fn a_read_only_source_refuses_to_move_and_says_so() {
    let mut r = rig_with(BoardSource::ProviderStatus, None, Duration::ZERO);
    paint(&mut r.view);
    r.view.on_key(shift(KeyCode::Right));
    assert!(r.view.moves.is_empty());
    assert!(r
        .view
        .message
        .as_deref()
        .unwrap_or("")
        .contains("read-only"));
    let text = paint(&mut r.view);
    assert!(text.contains("Read-only"), "the board says so:\n{text}");
    assert!(r.shared.lock().unwrap().moves.is_empty());
}

#[test]
fn the_filters_narrow_the_cards_and_clear_again() {
    let mut r = rig();
    r.shared.lock().unwrap().issues[1].assignees = vec!["Ines Doe".into()];
    r.shared.lock().unwrap().issues[2].labels = vec!["ui".into()];
    r.view.refresh();
    wait_for(&mut r.view, |v| v.load_rx.is_none());
    r.view
        .set_filter(FilterKind::Assignee, Some("Ines Doe".into()));
    assert_eq!(column_numbers(&r.view, 0), [2]);
    assert!(column_numbers(&r.view, 1).is_empty());
    r.view.clear_filters();
    r.view.set_filter(FilterKind::Label, Some("ui".into()));
    assert_eq!(column_numbers(&r.view, 1), [3]);
    assert!(column_numbers(&r.view, 0).is_empty());
    // The text filter is typed after `/`.
    r.view.clear_filters();
    r.view.on_key(key(KeyCode::Char('/')));
    for c in "number 4".chars() {
        r.view.on_key(key(KeyCode::Char(c)));
    }
    assert_eq!(column_numbers(&r.view, 2), [4]);
    assert!(column_numbers(&r.view, 0).is_empty());
    r.view.on_key(key(KeyCode::Enter));
    assert!(!r.view.searching);
    r.view.on_key(key(KeyCode::Char('x')));
    assert_eq!(column_numbers(&r.view, 0), [1, 2]);
}

#[test]
fn the_filter_dropdown_offers_all_then_the_values_the_cards_carry() {
    let mut r = rig();
    paint(&mut r.view);
    r.view.on_key(key(KeyCode::Char('l')));
    paint(&mut r.view);
    let combo = &r.view.combos[FilterKind::Label.index()];
    assert!(combo.is_open());
    let labels: Vec<&str> = combo.options.iter().map(|o| o.label.as_str()).collect();
    assert_eq!(labels, ["All labels", "bug"]);
}

#[test]
fn new_card_asks_for_the_form_in_the_selected_column() {
    let mut r = rig();
    paint(&mut r.view);
    r.view.on_key(key(KeyCode::Right));
    match r.view.on_key(key(KeyCode::Char('n'))) {
        BoardEffect::NewIssue(seed, at) => {
            assert_eq!(seed.project, "g/p");
            assert_eq!(at.column.as_deref(), Some("doing"));
            assert_eq!(at.label, None);
        }
        other => panic!("{other:?}"),
    }
}

#[test]
fn a_label_board_starts_the_new_issue_with_the_columns_label() {
    let mut r = rig_with(BoardSource::Labels, None, Duration::ZERO);
    r.view.target.labels = vec!["status::todo".into(), "status::doing".into()];
    r.view.spec.labels = r.view.target.labels.clone();
    r.view.rebuild();
    // Open, status::todo, status::doing, Closed.
    r.view.select(2, None);
    match r.view.run(BoardCmd::NewCard) {
        BoardEffect::NewIssue(_, at) => {
            assert_eq!(at.label.as_deref(), Some("status::doing"));
            assert_eq!(at.column, None);
        }
        other => panic!("{other:?}"),
    }
}

#[test]
fn too_many_columns_scroll_horizontally_with_a_bar_and_the_selection_stays_in_view() {
    // The layout: as many columns as fit, the bar when not all do.
    assert_eq!(layout(3, 130).0, 3);
    let (visible, each) = layout(8, 100);
    assert!(visible < 8 && each >= 26, "{visible} x {each}");
    let mut r = rig();
    {
        let mut s = r.shared.lock().unwrap();
        for (k, status) in ["a", "b", "c", "d", "e"].iter().enumerate() {
            s.issues.push(issue(10 + k as u64, status));
        }
    }
    r.view.refresh();
    wait_for(&mut r.view, |v| v.load_rx.is_none());
    let mut buf = Buffer::empty(Rect::new(0, 0, 90, 30));
    r.view.render(Rect::new(0, 0, 90, 30), &mut buf);
    assert!(r.view.visible_cols < r.view.columns.len());
    assert!(r.view.hbar.is_some(), "the horizontal bar is painted");
    for _ in 0..7 {
        r.view.on_key(key(KeyCode::Right));
    }
    r.view.render(Rect::new(0, 0, 90, 30), &mut buf);
    assert!(
        r.view.sel_col >= r.view.first_col
            && r.view.sel_col < r.view.first_col + r.view.visible_cols,
        "the selected column {} is outside {}..+{}",
        r.view.sel_col,
        r.view.first_col,
        r.view.visible_cols
    );
    // The bar's end arrow scrolls one column, like the keyboard would.
    let track = r.view.hbar.expect("bar");
    let before = r.view.first_col;
    r.view.on_mouse(mouse(
        MouseEventKind::Down(MouseButton::Left),
        track.rect.x,
        track.rect.y,
    ));
    assert_eq!(r.view.first_col, before.saturating_sub(1));
}

/// The cause of the `an_empty_column_is_still_shown` flake: the counter the
/// board watches was one process-wide static, so another test's bump (this
/// file's own `a_change_the_sidebar_noticed...`, the sidebar tests) made a
/// board that was mid-test start a second load, which read the fixture
/// before the test edited it. In unit tests the counter is per thread.
#[test]
fn a_bump_in_another_test_does_not_reach_this_board() {
    let before = crate::hosting_ui::issues_generation();
    std::thread::spawn(crate::hosting_ui::bump_issues_generation_for_tests)
        .join()
        .unwrap();
    assert_eq!(crate::hosting_ui::issues_generation(), before);
    // The board's own thread still sees its own bump.
    crate::hosting_ui::bump_issues_generation_for_tests();
    assert_eq!(crate::hosting_ui::issues_generation(), before + 1);
}

#[test]
fn a_change_the_sidebar_noticed_makes_the_board_read_again() {
    let mut r = rig();
    r.shared.lock().unwrap().issues.push(issue(9, "doing"));
    assert_eq!(column_numbers(&r.view, 1), [3]);
    crate::hosting_ui::bump_issues_generation_for_tests();
    wait_for(&mut r.view, |v| v.cards[1].len() == 2);
    assert_eq!(column_numbers(&r.view, 1).len(), 2);
}

#[test]
fn the_palette_commands_say_why_they_cannot_run() {
    let mut r = rig_with(BoardSource::ProviderStatus, None, Duration::ZERO);
    paint(&mut r.view);
    let avail = Some(r.view.avail());
    assert!(BoardCmd::MoveTo.unavailable(avail).is_some());
    assert!(BoardCmd::MoveRight.unavailable(avail).is_some());
    assert!(BoardCmd::OpenCard.unavailable(avail).is_none());
    assert!(BoardCmd::ClearFilters.unavailable(avail).is_some());
    assert!(BoardCmd::SetColumnLabels.unavailable(avail).is_some());
    assert!(BoardCmd::MoveTo.unavailable(None).is_some());
}

#[test]
fn dragging_onto_an_empty_column_paints_and_drops() {
    let mut r = rig();
    r.shared.lock().unwrap().issues.retain(|i| i.number != 4);
    r.view.refresh();
    wait_for(&mut r.view, |v| v.load_rx.is_none());
    let area = Rect::new(0, 0, 160, 60);
    let mut buf = Buffer::empty(area);
    r.view.render(area, &mut buf);
    let card = hit_rect(&r.view, Hit::Card(0, 0));
    let target = hit_rect(&r.view, Hit::Header(2));
    r.view.on_mouse(mouse(
        MouseEventKind::Down(MouseButton::Left),
        card.x + 1,
        card.y + 1,
    ));
    for step in 0..=20u16 {
        r.view.on_mouse(mouse(
            MouseEventKind::Drag(MouseButton::Left),
            card.x + 1 + step * (target.x + 2 - card.x - 1) / 20,
            card.y + 1 + step * 4 / 20,
        ));
        r.view.render(area, &mut buf);
    }
    r.view.on_mouse(mouse(
        MouseEventKind::Up(MouseButton::Left),
        target.x + 2,
        target.y + 5,
    ));
    assert_eq!(column_numbers(&r.view, 2), [1]);
}

#[test]
fn the_column_labels_prompt_saves_the_list_for_the_repository_and_the_board_follows() {
    let mut r = rig_with(BoardSource::Labels, None, Duration::ZERO);
    assert_eq!(
        r.view
            .columns
            .iter()
            .map(|c| c.name.as_str())
            .collect::<Vec<_>>(),
        ["Open", "Closed"],
        "no label set: Open and Closed only"
    );
    paint(&mut r.view);
    r.view.on_key(key(KeyCode::Char('c')));
    assert!(!r.view.takes_palette(), "the prompt owns the keyboard");
    let text = paint(&mut r.view);
    assert!(text.contains("Column labels"), "{text}");
    for c in "status::todo, status::doing".chars() {
        r.view.on_key(key(KeyCode::Char(c)));
    }
    r.view.on_key(key(KeyCode::Enter));
    assert!(r.view.takes_palette());
    let names: Vec<String> = r.view.columns.iter().map(|c| c.name.clone()).collect();
    assert_eq!(names, ["Open", "status::todo", "status::doing", "Closed"]);
    let state =
        gitpoulpi_session::RepoState::load(&crate::remotes_ui::repo_state_path(r.dir.path()))
            .unwrap();
    assert_eq!(state.board_labels, ["status::todo", "status::doing"]);
}

#[test]
fn the_card_menu_lists_the_actions_and_move_to_opens_the_column_dropdown() {
    let mut r = rig();
    paint(&mut r.view);
    r.view.on_key(key(KeyCode::Char('.')));
    paint(&mut r.view);
    let (_, menu) = r.view.menu.as_ref().expect("the card menu");
    let labels: Vec<&str> = menu.options.iter().map(|o| o.label.as_str()).collect();
    assert_eq!(
        labels,
        [
            "Open",
            "Edit assignees...",
            "Edit labels...",
            "Set milestone...",
            "Move to...",
            "Copy reference"
        ]
    );
    for _ in 0..4 {
        r.view.on_key(key(KeyCode::Down));
    }
    r.view.on_key(key(KeyCode::Enter));
    assert!(r.view.menu.is_none());
    assert!(r.view.move_combo.is_some(), "Move to... opens the dropdown");
    // `e` is the edit-only twin.
    r.view.on_key(key(KeyCode::Esc));
    r.view.on_key(key(KeyCode::Char('e')));
    let (kind, menu) = r.view.menu.as_ref().expect("the edit menu");
    assert_eq!(*kind, MenuKind::Edit);
    assert_eq!(menu.options.len(), 3);
}

#[test]
fn a_jira_move_with_no_transition_snaps_back_with_the_providers_reason() {
    let reason = "no transition from todo to doing for ABC-1";
    let mut r = rig_with(BoardSource::LocalStatus, Some(reason), Duration::ZERO);
    r.view.on_key(shift(KeyCode::Right));
    wait_for(&mut r.view, |v| v.moves.is_empty());
    assert_eq!(column_numbers(&r.view, 0), [1, 2], "the card is back");
    let text = paint(&mut r.view);
    assert!(text.contains(reason), "{text}");
}

#[test]
fn a_ctrl_alt_chord_is_ignored_but_the_normalized_altgr_key_acts() {
    use crossterm::event::{Event, KeyModifiers};
    let mut r = rig();
    paint(&mut r.view);
    // What the Windows console reports for a character typed with AltGr.
    let altgr = KeyEvent::new(
        KeyCode::Char('.'),
        KeyModifiers::CONTROL | KeyModifiers::ALT,
    );
    r.view.on_key(altgr);
    assert!(r.view.menu.is_none(), "a Ctrl+Alt chord is not a plain key");
    let Event::Key(plain) = crate::keymap::normalize_event(Event::Key(altgr)) else {
        panic!("a key stays a key")
    };
    r.view.on_key(plain);
    paint(&mut r.view);
    assert!(
        r.view.menu.is_some(),
        "the normalized key opens the card menu"
    );
}

#[test]
fn a_column_with_more_cards_than_room_has_a_bar_the_thumb_drags_and_the_window_stays_put() {
    let mut r = rig();
    {
        let mut s = r.shared.lock().unwrap();
        for n in 0..30u64 {
            s.issues.push(issue(100 + n, "todo"));
        }
    }
    r.view.refresh();
    wait_for(&mut r.view, |v| v.load_rx.is_none());
    let mut buf = Buffer::empty(AREA);
    r.view.render(AREA, &mut buf);
    assert_eq!(r.view.col_bars.len(), 1, "only the long column has a bar");
    let (col, track) = r.view.col_bars[0];
    assert_eq!(col, 0);
    assert_eq!(buf[(track.rect.x, track.rect.y)].symbol(), "\u{2191}");
    assert_eq!(
        buf[(track.rect.x, track.rect.bottom() - 1)].symbol(),
        "\u{2193}"
    );
    let selected = r.view.sel_number;
    // The thumb dragged past the bottom: the column is at its end, nothing
    // else moved, and the next paints keep it there.
    let x = track.rect.x;
    r.view.on_mouse(mouse(
        MouseEventKind::Down(MouseButton::Left),
        x,
        track.rect.y + track.thumb_start,
    ));
    r.view.on_mouse(mouse(
        MouseEventKind::Drag(MouseButton::Left),
        x,
        track.rect.bottom() + 5,
    ));
    r.view
        .on_mouse(mouse(MouseEventKind::Up(MouseButton::Left), x, 0));
    let max = r.view.cards[0].len() - r.view.rows_per_col;
    assert_eq!(r.view.offsets[0], max);
    r.view.render(AREA, &mut buf);
    r.view.render(AREA, &mut buf);
    assert_eq!(r.view.offsets[0], max, "a paint does not undo the drag");
    assert_eq!(r.view.sel_number, selected, "the selection did not move");
    // The wheel over the column scrolls it back, and so does the up arrow.
    let at = r.view.col_bars[0].1.rect;
    r.view
        .on_mouse(mouse(MouseEventKind::ScrollUp, at.x - 5, at.y + 1));
    r.view.render(AREA, &mut buf);
    assert_eq!(r.view.offsets[0], max - 1);
    r.view
        .on_mouse(mouse(MouseEventKind::Down(MouseButton::Left), x, at.y));
    r.view
        .on_mouse(mouse(MouseEventKind::Up(MouseButton::Left), x, at.y));
    r.view.render(AREA, &mut buf);
    assert_eq!(r.view.offsets[0], max - 2);
    // A key that moves the selection brings the column back to it.
    r.view.on_key(key(KeyCode::Down));
    r.view.render(AREA, &mut buf);
    let row = r.view.sel_row().expect("a selected card");
    assert!(r.view.offsets[0] <= row && row < r.view.offsets[0] + r.view.rows_per_col);
}

#[test]
fn a_short_column_has_no_bar() {
    let mut r = rig();
    paint(&mut r.view);
    assert!(r.view.col_bars.is_empty());
}

/// A repository that blocks assignment: the board's "Edit assignees" is
/// grayed with the same reason as the forms and the palette, and the key
/// says so instead of opening the picker. The other metadata stays.
#[test]
fn a_blocked_repository_grays_the_assign_command_with_the_reason() {
    const WHY: &str = "assignment is blocked in this repository";
    let mut r = rig();
    wait_for(&mut r.view, |v| v.load_rx.is_none());
    paint(&mut r.view);
    gitpoulpi_daemon::hosting_scope::register(r.dir.path(), "corp", "g/p");
    let open = Some(r.view.avail());
    assert_ne!(BoardCmd::EditAssignees.unavailable(open), Some(WHY));

    let state_dir = gitpoulpi_config::Settings::resolve_ambient()
        .map(|s| s.config().session.per_repo_state_dir.clone())
        .unwrap_or_else(|_| gitpoulpi_config::DEFAULT_REPO_DATA_LOCATION.to_string());
    gitpoulpi_session::RepoState {
        assignment_blocked: true,
        ..Default::default()
    }
    .save(&gitpoulpi_session::RepoState::path_for(
        r.dir.path(),
        &state_dir,
    ))
    .unwrap();
    let blocked = Some(r.view.avail());
    assert_eq!(BoardCmd::EditAssignees.unavailable(blocked), Some(WHY));
    assert_ne!(BoardCmd::EditLabels.unavailable(blocked), Some(WHY));
    r.view.run(BoardCmd::EditAssignees);
    assert_eq!(r.view.message.as_deref(), Some(WHY));
}
