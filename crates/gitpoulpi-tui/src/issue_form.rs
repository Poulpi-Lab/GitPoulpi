//! New issue: the form behind the ISSUES header's `+` (and the palette's
//! `Hosting: New Issue...`), for the provider and repository the ISSUES
//! section is linked to. Title, a multi-line description and optional
//! labels (comma-separated); Create / Cancel by mouse, Ctrl+S or Enter on
//! the button, Esc. The host call runs on its own thread
//! ([`start_create`]), never blocking the input loop.
//!
//! Same shape as the pull request form (`pull_request.rs`), minus the
//! provider tabs and branch pickers: nothing here chooses a project, the
//! link already did.

use super::*;
use crate::keymap::Action as Bind;
use gitpoulpi_i18n::msg;

use std::sync::mpsc as std_mpsc;
use std::sync::Arc;

use crossterm::event::MouseEvent;
use gitpoulpi_hosting::{CreatedIssue, Integration, NewIssue};

use crate::combo::{Combo, ComboOption, ComboOutcome};
use crate::dialog_kit::{self, ButtonState};
use crate::hosting_task::{start_issue_options, ClientFactory, OptionsReceiver};
use crate::meta_picker::{self, MetaField, MetaFields, MetaTarget};

/// What opens the form: the linked provider profile and project.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct IssueSeed {
    pub(crate) integration: Integration,
    /// The profile's display name, shown in the form.
    pub(crate) provider: String,
    /// `owner/name`, `group/sub/proj`.
    pub(crate) project: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum IssueFocus {
    Title,
    /// The Type (Jira, Azure), List (Trello) or Status dropdown.
    Choice,
    Body,
    Labels,
    /// The metadata buttons (GitHub, GitLab): they replace the Labels text
    /// field.
    MetaAssignees,
    MetaLabels,
    MetaMilestone,
    Web,
    Create,
    Cancel,
}

/// A click target of the form.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum IssueHit {
    Part(IssueFocus),
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[allow(clippy::large_enum_variant)] // short-lived, one per submit
pub(crate) enum IssueOutcome {
    Stay,
    Cancel,
    /// "Create on GitHub / GitLab": open the provider's own prefilled
    /// form (the URL) in the browser; the dialog stays open.
    OpenWeb(String),
    Submit {
        integration: Integration,
        req: Box<NewIssue>,
    },
}

/// What the extra dropdown of the form chooses, by provider.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ChoiceKind {
    /// A Jira issue type or an Azure work item type.
    Type,
    /// A Trello list.
    List,
    /// The status of a local Markdown note (its board column), a Linear
    /// workflow state, a YouTrack `State`.
    Status,
}

impl ChoiceKind {
    fn of(family: gitpoulpi_config::ApiFamily) -> Option<Self> {
        use gitpoulpi_config::ApiFamily;
        match family {
            ApiFamily::Jira | ApiFamily::AzureDevops | ApiFamily::Redmine => Some(Self::Type),
            ApiFamily::Trello => Some(Self::List),
            ApiFamily::LocalMarkdown | ApiFamily::Linear | ApiFamily::YouTrack => {
                Some(Self::Status)
            }
            _ => None,
        }
    }

    fn label(self) -> &'static str {
        match self {
            Self::Type => msg::issue_form::FIELD_TYPE.get(),
            Self::List => msg::issue_form::FIELD_LIST.get(),
            Self::Status => gitpoulpi_i18n::msg::local_issues::FIELD_STATUS.get(),
        }
    }
}

/// The Type / List dropdown: its choices come from the host
/// (`issue_create_options`), fetched when the form opens.
struct ChoiceField {
    kind: ChoiceKind,
    combo: Combo,
    rx: Option<OptionsReceiver>,
    since: std::time::Instant,
}

/// A request in flight for longer than this shows a spinner (rule 9).
const SPIN_AFTER: std::time::Duration = std::time::Duration::from_millis(150);

pub(crate) struct IssueDialog {
    pub(crate) seed: IssueSeed,
    pub(crate) title: String,
    /// The description: the shared Markdown editor.
    pub(crate) md: crate::md_editor::MdEditor,
    pub(crate) labels: String,
    pub(crate) focus: IssueFocus,
    pub(crate) message: Option<String>,
    pub(crate) busy: bool,
    pub(crate) hover: Option<IssueHit>,
    hits: Vec<(Rect, IssueHit)>,
    edit: crate::text_edit::FieldEditor<IssueFocus>,
    choice: Option<ChoiceField>,
    /// Assignees, labels and milestone, chosen in the picker.
    pub(crate) meta: MetaFields,
    factory: Option<Arc<dyn ClientFactory>>,
    integrations: Vec<Integration>,
    /// The Type / List option (by its label) to select once the choices
    /// arrive: the board column the form was opened from.
    preselect: Option<String>,
    guard: dialog_kit::DiscardGuard,
    /// The title as the description's "Write with AI" help reads it.
    ai_title: Option<crate::md_editor::ai::Shared<String>>,
}

impl IssueDialog {
    /// What `Ctrl+N` badges: every field and button.
    pub(crate) fn nav_targets(&self) -> Vec<(Rect, (u16, u16))> {
        dialog_kit::nav_cells(self.hits.iter().map(|(r, _)| *r).chain(self.md.nav_rects()))
    }

    /// Gives the description its "Write with AI" help (only while AI is on).
    pub(crate) fn bind_ai(&mut self, scope: crate::md_editor::ai::AiScope) {
        let title = crate::md_editor::ai::Shared::default();
        self.md.bind_ai(
            scope,
            Box::new(crate::md_editor::ai::IssueBodySource {
                title: title.clone(),
            }),
        );
        self.ai_title = Some(title);
    }

    pub(crate) fn new(seed: IssueSeed) -> Self {
        let draft_key =
            crate::md_editor::draft::key(&[&seed.integration.id, &seed.project, "new-issue"]);
        let md_context = crate::md_render::Context {
            xrefs: gitpoulpi_hosting::xrefs::XrefContext {
                family: Some(seed.integration.kind.family()),
                web_base: gitpoulpi_hosting::integration_web_base(&seed.integration)
                    .unwrap_or_default(),
                project: seed.project.clone(),
                ..Default::default()
            },
        };
        let choice = ChoiceKind::of(seed.integration.kind.family()).map(|kind| ChoiceField {
            kind,
            combo: Combo::new(""),
            rx: None,
            since: std::time::Instant::now(),
        });
        Self {
            choice,
            seed,
            title: String::new(),
            md: crate::md_editor::MdEditor::new(String::new(), md_context).with_draft(draft_key),
            labels: String::new(),
            focus: IssueFocus::Title,
            message: None,
            busy: false,
            hover: None,
            hits: Vec::new(),
            edit: Default::default(),
            meta: MetaFields::default(),
            factory: None,
            integrations: Vec::new(),
            preselect: None,
            guard: Default::default(),
            ai_title: None,
        }
    }

    /// Starts the form in a board column: the status / list `column`
    /// names, and the status `label` the new issue carries (a host board).
    pub(crate) fn start_in(&mut self, column: Option<String>, label: Option<String>) {
        self.preselect = column;
        let Some(label) = label else {
            return;
        };
        if self.uses_meta() {
            self.meta.set_scope(format!(
                "{}|{}",
                self.seed.integration.id, self.seed.project
            ));
            self.meta.selection.labels.push(label);
        } else {
            self.labels = label;
        }
    }

    /// The provider has a web form to fall back on (not a local folder).
    fn has_web(&self) -> bool {
        !self.seed.integration.kind.capabilities().local_folder
    }

    /// Whether the form offers the metadata pickers (GitHub, GitLab, local
    /// Markdown).
    pub(crate) fn uses_meta(&self) -> bool {
        meta_picker::supports(self.seed.integration.kind)
    }

    /// The provider has labels to pick (Redmine has none).
    fn has_labels(&self) -> bool {
        meta_picker::supports_field(self.seed.integration.kind, MetaField::Labels)
    }

    fn meta_field(part: IssueFocus) -> Option<MetaField> {
        match part {
            IssueFocus::MetaAssignees => Some(MetaField::Assignees),
            IssueFocus::MetaLabels => Some(MetaField::Labels),
            IssueFocus::MetaMilestone => Some(MetaField::Milestone),
            _ => None,
        }
    }

    fn focus_of(field: MetaField) -> IssueFocus {
        match field {
            MetaField::Assignees | MetaField::Reviewers => IssueFocus::MetaAssignees,
            MetaField::Labels => IssueFocus::MetaLabels,
            MetaField::Milestone => IssueFocus::MetaMilestone,
        }
    }

    fn open_meta(&mut self, field: MetaField) {
        let Some(factory) = self.factory.clone() else {
            return;
        };
        self.meta.set_scope(format!(
            "{}|{}",
            self.seed.integration.id, self.seed.project
        ));
        let target = MetaTarget {
            integration: self.seed.integration.clone(),
            all: self.integrations.clone(),
            project: self.seed.project.clone(),
            kind: gitpoulpi_hosting::MetaKind::Issue,
            number: None,
            caption: msg::meta_picker::CREATE_ISSUE_CAPTION.get().to_string(),
        };
        self.meta.open(factory, target, field);
    }

    /// A picker is waiting on the host (the spinner moves).
    pub(crate) fn animating(&self) -> bool {
        self.meta.busy()
    }

    /// Starts asking the host for the Type / List choices (nothing to ask
    /// for GitHub and GitLab).
    pub(crate) fn fetch_options(&mut self, factory: Arc<dyn ClientFactory>, all: Vec<Integration>) {
        self.factory = Some(factory.clone());
        self.integrations = all.clone();
        let Some(choice) = self.choice.as_mut() else {
            return;
        };
        choice.since = std::time::Instant::now();
        choice.rx = Some(start_issue_options(
            factory,
            self.seed.integration.clone(),
            all,
            self.seed.project.clone(),
        ));
    }

    /// Drains the choices fetch. True when the form changed.
    pub(crate) fn poll(&mut self) -> bool {
        let picked = self.meta.poll();
        let Some(choice) = self.choice.as_mut() else {
            return picked;
        };
        let Some(answer) = choice.rx.as_ref().and_then(|rx| rx.try_recv()) else {
            return picked;
        };
        choice.rx = None;
        match answer {
            Ok(options) => {
                let (list, default) = match choice.kind {
                    ChoiceKind::Type => (options.types, options.default_type),
                    ChoiceKind::List | ChoiceKind::Status => (options.lists, options.default_list),
                };
                choice.combo.set_options(
                    list.into_iter()
                        .map(|c| ComboOption::new(c.label, c.id))
                        .collect(),
                );
                choice.combo.select(default);
                if let Some(wanted) = self.preselect.take() {
                    let found = choice
                        .combo
                        .options
                        .iter()
                        .find(|o| o.label.trim().eq_ignore_ascii_case(wanted.trim()))
                        .map(|o| o.value.clone());
                    if found.is_some() {
                        choice.combo.select(found);
                    }
                }
            }
            Err(why) => self.message = Some(why),
        }
        true
    }

    /// Whether this repository blocks assignment: the assignee picker is
    /// left out and nothing is assigned.
    fn assignment_blocked(&self) -> bool {
        gitpoulpi_hosting::scope::assignment_blocked(&self.seed.integration.id, &self.seed.project)
    }

    /// The focus order, top to bottom.
    fn order(&self) -> Vec<IssueFocus> {
        let mut order = vec![IssueFocus::Title];
        if self.choice.is_some() {
            order.push(IssueFocus::Choice);
        }
        order.push(IssueFocus::Body);
        if self.uses_meta() {
            if !self.assignment_blocked() {
                order.push(IssueFocus::MetaAssignees);
            }
            if self.has_labels() {
                order.push(IssueFocus::MetaLabels);
            }
            order.push(IssueFocus::MetaMilestone);
        } else {
            order.push(IssueFocus::Labels);
        }
        if self.has_web() {
            order.push(IssueFocus::Web);
        }
        order.extend([IssueFocus::Create, IssueFocus::Cancel]);
        order
    }

    /// The request the fields describe, or why there is none yet.
    pub(crate) fn plan(&self) -> Result<NewIssue, String> {
        let title = self.title.trim();
        if title.is_empty() {
            return Err(msg::issue_form::TYPE_TITLE.get().into());
        }
        Ok(NewIssue {
            project: self.seed.project.clone(),
            title: title.to_string(),
            body: self.md.text().trim().to_string(),
            labels: self.chosen_labels(),
            assignees: if self.uses_meta() && !self.assignment_blocked() {
                self.meta.selection.assignees.clone()
            } else {
                Vec::new()
            },
            milestone: if self.uses_meta() {
                self.meta.selection.milestone.as_ref().map(|m| m.id)
            } else {
                None
            },
            issue_type: self
                .choice
                .as_ref()
                .filter(|c| c.kind == ChoiceKind::Type)
                .and_then(|c| c.combo.selected().map(str::to_string)),
            list_id: self
                .choice
                .as_ref()
                .filter(|c| matches!(c.kind, ChoiceKind::List | ChoiceKind::Status))
                .and_then(|c| c.combo.selected().map(str::to_string)),
        })
    }

    /// The labels: the picker's on GitHub and GitLab, else the typed list.
    fn chosen_labels(&self) -> Vec<String> {
        if self.uses_meta() {
            return self.meta.selection.labels.clone();
        }
        self.labels
            .split(',')
            .map(|l| l.trim().to_string())
            .filter(|l| !l.is_empty())
            .collect()
    }

    /// The provider's name (`GitHub` / `GitLab`).
    pub(crate) fn provider_name(&self) -> &'static str {
        crate::hosting_ui::provider_name(self.seed.integration.kind)
    }

    /// The provider's own new-issue form, prefilled with the fields as
    /// typed (works without a token, whatever the API call did).
    pub(crate) fn web_url(&self) -> Option<String> {
        if !self.has_web() {
            return None;
        }
        let base = gitpoulpi_hosting::integration_web_base(&self.seed.integration)?;
        let labels = self.chosen_labels();
        Some(gitpoulpi_hosting::new_issue_form_url(
            self.seed.integration.kind,
            &base,
            &self.seed.project,
            &self.title,
            self.md.text(),
            &labels,
        ))
    }

    fn open_web(&mut self) -> IssueOutcome {
        match self.web_url() {
            Some(url) => IssueOutcome::OpenWeb(url),
            None => {
                self.message = Some(msg::issue_form::NO_WEB.get().into());
                IssueOutcome::Stay
            }
        }
    }

    fn submit(&mut self) -> IssueOutcome {
        if self.busy {
            return IssueOutcome::Stay;
        }
        match self.plan() {
            Ok(req) => IssueOutcome::Submit {
                integration: self.seed.integration.clone(),
                req: Box::new(req),
            },
            Err(reason) => {
                self.message = Some(reason);
                IssueOutcome::Stay
            }
        }
    }

    /// Something typed would be lost on close.
    fn typed(&self) -> bool {
        [self.title.as_str(), self.md.text(), self.labels.as_str()]
            .iter()
            .any(|t| !t.trim().is_empty())
    }

    /// The cross, Esc, Cancel and a click outside all ask the one rule.
    fn request_close(&mut self, kind: dialog_kit::DismissKind) -> IssueOutcome {
        use dialog_kit::{dismiss, DismissAction};
        match dismiss(&*self, kind) {
            DismissAction::Close => IssueOutcome::Cancel,
            DismissAction::Swallow => {
                self.guard.close.request_flash();
                IssueOutcome::Stay
            }
            DismissAction::Confirm => {
                self.guard.ask();
                IssueOutcome::Stay
            }
        }
    }

    fn verdict(&self, v: dialog_kit::Verdict) -> IssueOutcome {
        match v {
            dialog_kit::Verdict::Discard => IssueOutcome::Cancel,
            dialog_kit::Verdict::Stay => IssueOutcome::Stay,
        }
    }

    fn activate(&mut self, part: IssueFocus) -> IssueOutcome {
        match part {
            IssueFocus::Cancel => self.request_close(dialog_kit::DismissKind::Cancel),
            IssueFocus::Create => self.submit(),
            IssueFocus::Web => self.open_web(),
            other => {
                if let Some(field) = Self::meta_field(other) {
                    self.open_meta(field);
                }
                IssueOutcome::Stay
            }
        }
    }

    fn text_of(&mut self, focus: IssueFocus) -> Option<&mut String> {
        match focus {
            IssueFocus::Title => Some(&mut self.title),
            IssueFocus::Labels => Some(&mut self.labels),
            _ => None,
        }
    }

    fn is_text(focus: IssueFocus) -> bool {
        matches!(
            focus,
            IssueFocus::Title | IssueFocus::Body | IssueFocus::Labels
        )
    }

    pub(crate) fn paste(&mut self, text: &str) {
        if self.meta.paste(text) {
            return;
        }
        let focus = self.focus;
        if !Self::is_text(focus) || self.guard.is_open() {
            return;
        }
        if focus == IssueFocus::Body {
            self.md.paste(text);
            return;
        }
        let clean: String = text.chars().filter(|c| !c.is_control()).collect();
        self.edit.sync(Some(focus));
        let mut value = std::mem::take(self.text_of(focus).expect("a text field"));
        self.edit.caret_mut(focus).insert(&mut value, &clean);
        *self.text_of(focus).expect("a text field") = value;
    }

    pub(crate) fn on_key(&mut self, key: KeyEvent) -> IssueOutcome {
        let shift = key.modifiers.contains(KeyModifiers::SHIFT);
        let focus = self.focus;
        if let Some(v) = self.guard.key(key) {
            return self.verdict(v);
        }
        // An open picker is modal: it takes every key.
        if self.meta.is_open() {
            self.meta.on_key(key);
            return IssueOutcome::Stay;
        }
        // An open dropdown is modal: it takes every key.
        if let Some(choice) = self.choice.as_mut().filter(|c| c.combo.is_open()) {
            choice.combo.on_key(key);
            return IssueOutcome::Stay;
        }
        self.edit.sync(
            Self::is_text(focus)
                .then_some(focus)
                .filter(|f| *f != IssueFocus::Body),
        );
        match focus {
            _ if crate::keymap::current().is(Bind::DialogWeb, &key)
                && !(focus == IssueFocus::Body && self.md.takes_ai_key(&key)) =>
            {
                return self.open_web()
            }
            IssueFocus::Title | IssueFocus::Labels => {
                let mut value = std::mem::take(self.text_of(focus).expect("a text field"));
                let handled = self.edit.dialog_key(focus, &mut value, &key);
                *self.text_of(focus).expect("a text field") = value;
                if let Some((result, copied)) = handled {
                    if result == crate::text_edit::KeyEdit::Edited {
                        self.message = None;
                    }
                    if copied.is_some() {
                        self.message = copied;
                    }
                    return IssueOutcome::Stay;
                }
            }
            IssueFocus::Body => match self.md.on_key(key) {
                crate::md_editor::KeyUse::Handled => {
                    self.message = None;
                    return IssueOutcome::Stay;
                }
                crate::md_editor::KeyUse::Submit => return self.submit(),
                crate::md_editor::KeyUse::Pass => {}
            },
            _ => {}
        }
        // A focused, closed dropdown opens on Enter / Space / Down.
        if focus == IssueFocus::Choice && !matches!(key.code, KeyCode::Tab | KeyCode::BackTab) {
            if let Some(choice) = self.choice.as_mut() {
                if !matches!(key.code, KeyCode::Up)
                    && choice.combo.on_key(key) == ComboOutcome::Handled
                {
                    self.message = None;
                    return IssueOutcome::Stay;
                }
            }
        }
        match key.code {
            KeyCode::Esc => return self.request_close(dialog_kit::DismissKind::Esc),
            KeyCode::Tab | KeyCode::BackTab | KeyCode::Down | KeyCode::Up => {
                let forward =
                    matches!(key.code, KeyCode::Down) || (key.code == KeyCode::Tab && !shift);
                let order = self.order();
                let at = order.iter().position(|f| *f == focus).unwrap_or(0);
                let len = order.len();
                self.focus = order[if forward {
                    (at + 1) % len
                } else {
                    (at + len - 1) % len
                }];
            }
            _ if crate::keymap::current().is(Bind::DialogSubmit, &key) => return self.submit(),
            KeyCode::Enter => {
                return match focus {
                    // Enter in a one-line field moves on, like a form.
                    IssueFocus::Title | IssueFocus::Labels => {
                        self.focus = if focus == IssueFocus::Title {
                            if self.choice.is_some() {
                                IssueFocus::Choice
                            } else {
                                IssueFocus::Body
                            }
                        } else {
                            IssueFocus::Create
                        };
                        IssueOutcome::Stay
                    }
                    other => self.activate(other),
                };
            }
            KeyCode::Char(' ') if !Self::is_text(focus) => return self.activate(focus),
            _ => {}
        }
        IssueOutcome::Stay
    }

    fn field_mouse(&mut self, mouse: MouseEvent) -> bool {
        let focus = self.focus;
        let (id, text) = match focus {
            IssueFocus::Title => (Some(focus), self.title.clone()),
            IssueFocus::Labels => (Some(focus), self.labels.clone()),
            _ => (None, String::new()),
        };
        self.edit.sync(id);
        self.edit
            .dialog_mouse(id, &text, mouse.kind, mouse.column, mouse.row)
    }

    pub(crate) fn on_mouse(&mut self, mouse: MouseEvent) -> IssueOutcome {
        if let Some(v) = self.guard.mouse(mouse) {
            return self.verdict(v);
        }
        // An open picker owns the pointer, nothing behind it hovers.
        if self.meta.is_open() {
            self.meta.on_mouse(mouse);
            return IssueOutcome::Stay;
        }
        // An open dropdown is modal: it takes every mouse event.
        if let Some(choice) = self.choice.as_mut().filter(|c| c.combo.is_open()) {
            choice.combo.on_mouse(mouse);
            return IssueOutcome::Stay;
        }
        if let Some(kind) = self.guard.pointer(mouse) {
            return self.request_close(kind);
        }
        if self.md.on_mouse(mouse) == crate::md_editor::MouseUse::Inside {
            if matches!(mouse.kind, MouseEventKind::Down(MouseButton::Left)) {
                self.focus = IssueFocus::Body;
            }
            return IssueOutcome::Stay;
        }
        let hit = dialog_kit::hit_at(&self.hits, mouse.column, mouse.row);
        if !matches!(mouse.kind, MouseEventKind::Down(_)) && self.field_mouse(mouse) {
            return IssueOutcome::Stay;
        }
        match mouse.kind {
            MouseEventKind::Moved | MouseEventKind::Drag(_) => self.hover = hit,
            MouseEventKind::Down(MouseButton::Left) => {
                if let Some(IssueHit::Part(part)) = hit {
                    self.focus = part;
                    if part == IssueFocus::Choice {
                        if let Some(choice) = self.choice.as_mut() {
                            choice.combo.on_mouse(mouse);
                        }
                    } else if Self::is_text(part) && part != IssueFocus::Body {
                        self.field_mouse(mouse);
                    } else {
                        return self.activate(part);
                    }
                }
            }
            _ => {}
        }
        IssueOutcome::Stay
    }

    pub(crate) fn render(&mut self, rect: Rect, buf: &mut Buffer) {
        self.edit.sync(
            Self::is_text(self.focus)
                .then_some(self.focus)
                .filter(|f| *f != IssueFocus::Body),
        );
        self.edit.begin_frame();
        if let Some(cell) = &self.ai_title {
            if let Ok(mut t) = cell.lock() {
                t.clone_from(&self.title);
            }
        }
        let title = match self.seed.integration.kind.family() {
            gitpoulpi_config::ApiFamily::AzureDevops => msg::issue_form::TITLE_WORK_ITEM.get(),
            gitpoulpi_config::ApiFamily::Trello => msg::issue_form::TITLE_CARD.get(),
            gitpoulpi_config::ApiFamily::LocalMarkdown => {
                gitpoulpi_i18n::msg::local_issues::NEW_NOTE_TITLE.get()
            }
            _ => msg::issue_form::TITLE.get(),
        };
        self.guard.framed(buf, rect, title, &WIN);
        let mut hits: Vec<(Rect, IssueHit)> = Vec::new();
        let inner = Rect::new(
            rect.x + 2,
            rect.y + 1,
            rect.width.saturating_sub(4),
            rect.height.saturating_sub(2),
        );
        if inner.width < 30 || inner.height < 12 {
            self.hits = hits;
            return;
        }
        let muted = Style::default().fg(th().ui.muted).bg(th().ui.panel);
        let mut y = inner.y;
        let target = format!("{} \u{2192} {}", self.seed.provider, self.seed.project);
        // "Create on GitHub": the provider's own form, top right (never
        // lost to a long footer, and still there after an API error).
        let web_label = msg::issue_form::create_on_host(self.provider_name());
        let web_w = if self.has_web() {
            dialog_kit::button_width(&web_label)
        } else {
            0
        };
        if self.has_web() {
            let web = dialog_kit::button(
                buf,
                inner.right().saturating_sub(web_w),
                y,
                &web_label,
                ButtonState::Normal {
                    hovered: self.hover == Some(IssueHit::Part(IssueFocus::Web)),
                    focused: self.focus == IssueFocus::Web,
                },
            );
            hits.insert(0, (web, IssueHit::Part(IssueFocus::Web)));
        }
        dialog_kit::put(
            buf,
            inner.x,
            y,
            &target,
            inner.width.saturating_sub(web_w + 1),
            Style::default().fg(th().ui.accent).bg(th().ui.panel),
        );
        y += 2;

        dialog_kit::put(
            buf,
            inner.x,
            y,
            msg::issue_form::FIELD_TITLE.get(),
            inner.width,
            muted,
        );
        y += 1;
        let title = Rect::new(inner.x, y, inner.width, 1);
        dialog_kit::text_input(
            buf,
            title,
            &self.title,
            msg::issue_form::TITLE_PLACEHOLDER.get(),
            self.focus == IssueFocus::Title,
            self.hover == Some(IssueHit::Part(IssueFocus::Title)),
            (&self.edit, IssueFocus::Title),
        );
        hits.push((title, IssueHit::Part(IssueFocus::Title)));
        y += 2;

        // The Type / List dropdown, once the form has one (Jira, Azure,
        // Trello): the label, then the host's choices.
        if let Some(choice) = self.choice.as_mut() {
            let label_w = 10;
            dialog_kit::put(buf, inner.x, y, choice.kind.label(), label_w, muted);
            let field = Rect::new(
                inner.x + label_w,
                y,
                inner.width.saturating_sub(label_w).min(34),
                1,
            );
            let note = if choice.rx.is_some() {
                if choice.since.elapsed() >= SPIN_AFTER {
                    format!(
                        "{} {}",
                        crate::activity::spinner_frame(choice.since.elapsed()),
                        msg::issue_form::OPTIONS_LOADING.get()
                    )
                } else {
                    String::new()
                }
            } else {
                msg::issue_form::OPTIONS_NONE.get().to_string()
            };
            choice.combo.set_placeholder(&note);
            choice.combo.render_field(
                buf,
                field,
                self.focus == IssueFocus::Choice,
                self.hover == Some(IssueHit::Part(IssueFocus::Choice)),
            );
            hits.push((field, IssueHit::Part(IssueFocus::Choice)));
            y += 2;
        }

        dialog_kit::put(
            buf,
            inner.x,
            y,
            msg::issue_form::FIELD_DESCRIPTION.get(),
            inner.width,
            muted,
        );
        y += 1;
        // Title, labels and the footer take their rows; the description
        // gets the rest.
        let footer_rows = 5; // labels label + field, spacing, message, buttons
                             // At most 10 rows at the default height; a taller dialog gives the
                             // rows it gains to the description.
        let body_cap = 10 + rect.height.saturating_sub(DEFAULT_HEIGHT);
        let body_h = inner
            .bottom()
            .saturating_sub(y + footer_rows)
            .clamp(2, body_cap);
        let body = Rect::new(inner.x, y, inner.width, body_h);
        self.md.render(
            buf,
            body,
            self.focus == IssueFocus::Body,
            msg::issue_form::DESCRIPTION_PLACEHOLDER.get(),
        );
        hits.push((body, IssueHit::Part(IssueFocus::Body)));
        y += body_h + 1;

        if self.uses_meta() {
            // GitHub and GitLab: assignees, labels and milestone are picked
            // from what the project has, in one row of buttons.
            let fields: Vec<MetaField> = [
                MetaField::Assignees,
                MetaField::Labels,
                MetaField::Milestone,
            ]
            .into_iter()
            .filter(|f| meta_picker::supports_field(self.seed.integration.kind, *f))
            .filter(|f| *f != MetaField::Assignees || !self.assignment_blocked())
            .collect();
            let focus = Self::meta_field(self.focus);
            let hover = match self.hover {
                Some(IssueHit::Part(part)) => Self::meta_field(part),
                None => None,
            };
            for (rect, field) in
                self.meta
                    .render_chips(buf, (inner.x, y), inner.width, &fields, focus, hover)
            {
                hits.push((rect, IssueHit::Part(Self::focus_of(field))));
            }
            if self.assignment_blocked() {
                dialog_kit::put(
                    buf,
                    inner.x,
                    y + 1,
                    msg::hosting_ui::WHY_ASSIGNMENT_BLOCKED.get(),
                    inner.width,
                    muted,
                );
            }
        } else {
            dialog_kit::put(
                buf,
                inner.x,
                y,
                msg::issue_form::FIELD_LABELS.get(),
                inner.width,
                muted,
            );
            y += 1;
            let labels = Rect::new(inner.x, y, inner.width, 1);
            dialog_kit::text_input(
                buf,
                labels,
                &self.labels,
                msg::issue_form::LABELS_PLACEHOLDER.get(),
                self.focus == IssueFocus::Labels,
                self.hover == Some(IssueHit::Part(IssueFocus::Labels)),
                (&self.edit, IssueFocus::Labels),
            );
            hits.push((labels, IssueHit::Part(IssueFocus::Labels)));
        }

        let bottom = inner.y + inner.height;
        let msg_y = bottom.saturating_sub(2);
        if let Some(message) = &self.message {
            dialog_kit::put(
                buf,
                inner.x,
                msg_y,
                message,
                inner.width,
                Style::default().fg(th().status.danger).bg(th().ui.panel),
            );
        } else if self.busy {
            dialog_kit::put(
                buf,
                inner.x,
                msg_y,
                msg::issue_form::CREATING.get(),
                inner.width,
                Style::default().fg(th().ui.accent).bg(th().ui.panel),
            );
        } else if self.hover == Some(IssueHit::Part(IssueFocus::Web)) {
            // The hover tooltip (the global one is off under a dialog).
            dialog_kit::put(
                buf,
                inner.x,
                msg_y,
                &msg::issue_form::tip_web(self.provider_name()),
                inner.width,
                muted,
            );
        } else if let Err(reason) = self.plan() {
            dialog_kit::put(
                buf,
                inner.x,
                msg_y,
                &reason,
                inner.width,
                Style::default().fg(th().status.warning).bg(th().ui.panel),
            );
        }

        let button_y = bottom.saturating_sub(1);
        let cancel_w = dialog_kit::button_width(msg::issue_form::CANCEL.get());
        let create_label = msg::issue_form::CREATE.get();
        let create_w = dialog_kit::button_width(create_label);
        let create_x = inner.right().saturating_sub(create_w);
        let cancel_x = create_x.saturating_sub(cancel_w + 1);
        let cancel = dialog_kit::button(
            buf,
            cancel_x,
            button_y,
            msg::issue_form::CANCEL.get(),
            ButtonState::Normal {
                hovered: self.hover == Some(IssueHit::Part(IssueFocus::Cancel)),
                focused: self.focus == IssueFocus::Cancel,
            },
        );
        hits.push((cancel, IssueHit::Part(IssueFocus::Cancel)));
        let create = dialog_kit::button(
            buf,
            create_x,
            button_y,
            create_label,
            if self.busy || self.plan().is_err() {
                ButtonState::Disabled
            } else {
                ButtonState::Primary {
                    hovered: self.hover == Some(IssueHit::Part(IssueFocus::Create)),
                    focused: self.focus == IssueFocus::Create,
                }
            },
        );
        hits.push((create, IssueHit::Part(IssueFocus::Create)));
        let create_key = dialog_kit::key_label(Bind::DialogSubmit);
        let web_key = dialog_kit::key_label(Bind::DialogWeb);
        let [(prev_key, prev_what), _] = crate::md_editor::MdEditor::hints();
        dialog_kit::hints(
            buf,
            inner.x,
            button_y,
            cancel_x.saturating_sub(inner.x + 1),
            &[
                (
                    msg::issue_form::KEY_TAB.get(),
                    msg::issue_form::HINT_NEXT.get(),
                ),
                (create_key.as_str(), msg::issue_form::HINT_CREATE.get()),
                (web_key.as_str(), msg::issue_form::HINT_WEB.get()),
                (prev_key.as_str(), prev_what),
            ],
        );
        self.hits = hits;
        // The open dropdown last, over everything else.
        if let Some(choice) = self.choice.as_mut() {
            choice.combo.render_popup(buf, buf.area);
        }
        self.meta.render_popup(buf);
        self.guard.render(buf);
    }

    #[cfg(all(test, feature = "github", feature = "gitlab"))]
    fn hit_rect(&self, part: IssueFocus) -> Rect {
        self.hits
            .iter()
            .find(|(_, h)| *h == IssueHit::Part(part))
            .map(|(r, _)| *r)
            .unwrap_or_else(|| panic!("{part:?} not drawn"))
    }
}

/// The rectangle the form draws in, centered like every other repo dialog.
pub(crate) fn issue_rect(area: Rect) -> Rect {
    WIN.place(
        area,
        dialog_kit::centered(area, area.width.saturating_sub(4).min(80), DEFAULT_HEIGHT),
    )
}

/// Resizable: the description grows with the height.
const WIN: dialog_kit::Win = dialog_kit::Win::resizable("issue", 50, 18);

/// The form's default height.
const DEFAULT_HEIGHT: u16 = 22;

impl dialog_kit::Dismiss for IssueDialog {
    fn is_dirty(&self) -> bool {
        self.typed()
    }

    fn guard(&self) -> dialog_kit::Guard {
        dialog_kit::Guard::ConfirmWhenDirty
    }

    fn close_hit(&self) -> &dialog_kit::CloseHit {
        &self.guard.close
    }

    fn close_hit_mut(&mut self) -> &mut dialog_kit::CloseHit {
        &mut self.guard.close
    }
}

// -- the background task ---------------------------------------------------

/// What a running `create_issue` call reports.
pub(crate) enum IssueEvent {
    Finished(Result<CreatedIssue, String>),
}

pub(crate) struct IssueReceiver {
    rx: std_mpsc::Receiver<IssueEvent>,
}

impl IssueReceiver {
    pub(crate) fn try_recv(&self) -> Option<IssueEvent> {
        self.rx.try_recv().ok()
    }
}

/// Starts the API call on its own thread, reporting on a channel the
/// render loop drains with `try_recv`.
pub(crate) fn start_create(
    factory: Arc<dyn ClientFactory>,
    integration: Integration,
    all: Vec<Integration>,
    req: NewIssue,
) -> IssueReceiver {
    let (tx, rx) = std_mpsc::channel();
    std::thread::spawn(move || {
        let result = (|| {
            let runtime = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .map_err(crate::hosting_task::request_task_failed)?;
            let client = factory
                .client(&integration, &all)
                .map_err(|e| crate::hosting_task::error_text(&e))?;
            runtime
                .block_on(client.create_issue(&req))
                .map_err(|e| crate::hosting_task::error_text(&e))
        })();
        let _ = tx.send(IssueEvent::Finished(result));
    });
    IssueReceiver { rx }
}

#[cfg(test)]
#[cfg(all(feature = "github", feature = "gitlab"))]
mod tests {
    use super::*;
    use gitpoulpi_hosting::IntegrationKind;
    use ratatui::buffer::Buffer;

    fn seed() -> IssueSeed {
        IssueSeed {
            integration: Integration::new("gl", IntegrationKind::GitlabSelfManaged, "http://h"),
            provider: "corp".into(),
            project: "g/p".into(),
        }
    }

    fn key(code: KeyCode) -> KeyEvent {
        KeyEvent::new(code, KeyModifiers::NONE)
    }

    fn type_text(form: &mut IssueDialog, text: &str) {
        for c in text.chars() {
            form.on_key(key(KeyCode::Char(c)));
        }
    }

    /// A tracker without metadata pickers: labels are typed.
    fn trello_seed() -> IssueSeed {
        IssueSeed {
            integration: Integration::new("tr", IntegrationKind::Trello, "https://trello.com"),
            provider: "board".into(),
            project: "g/p".into(),
        }
    }

    /// A repository that blocks assignment: the form has no assignee picker,
    /// says why, and sends no assignee even if one was chosen before.
    #[test]
    fn a_blocked_repository_has_no_assignee_picker_in_the_issue_form() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        let state_dir = gitpoulpi_config::Settings::resolve_ambient()
            .map(|s| s.config().session.per_repo_state_dir.clone())
            .unwrap_or_else(|_| gitpoulpi_config::DEFAULT_REPO_DATA_LOCATION.to_string());
        let path = gitpoulpi_session::RepoState::path_for(root, &state_dir);
        let seed = IssueSeed {
            integration: Integration::new(
                "blocked-form",
                IntegrationKind::GitlabSelfManaged,
                "http://h",
            ),
            provider: "corp".into(),
            project: "blocked/form".into(),
        };
        gitpoulpi_daemon::hosting_scope::register(root, "blocked-form", "blocked/form");
        let mut form = IssueDialog::new(seed);
        form.meta.selection.apply(meta_picker::Applied::List(
            MetaField::Assignees,
            vec!["ann".into()],
        ));
        type_text(&mut form, "T");
        // Allowed (no file): the picker is there and the choice is sent.
        assert!(form.order().contains(&IssueFocus::MetaAssignees));
        assert_eq!(form.plan().unwrap().assignees, ["ann"]);

        gitpoulpi_session::RepoState {
            assignment_blocked: true,
            ..Default::default()
        }
        .save(&path)
        .unwrap();
        assert!(!form.order().contains(&IssueFocus::MetaAssignees));
        assert!(form.plan().unwrap().assignees.is_empty());
        let area = Rect::new(0, 0, 100, 30);
        let mut buf = Buffer::empty(area);
        form.render(issue_rect(area), &mut buf);
        let text = crate::dialog_kit::buffer_text(&buf);
        assert!(!text.contains("Assignees"), "{text}");
        assert!(
            text.contains("assignment is blocked in this repository"),
            "{text}"
        );
        assert!(
            text.contains("Labels: none"),
            "the other pickers stay:\n{text}"
        );
    }

    #[test]
    fn a_redmine_form_has_a_tracker_dropdown_and_no_labels_button() {
        let seed = IssueSeed {
            integration: Integration::new("rm", IntegrationKind::Redmine, "http://h"),
            provider: "acme".into(),
            project: "bench".into(),
        };
        let mut form = IssueDialog::new(seed);
        let order = form.order();
        assert!(order.contains(&IssueFocus::Choice), "the tracker dropdown");
        assert!(order.contains(&IssueFocus::MetaAssignees));
        assert!(order.contains(&IssueFocus::MetaMilestone));
        assert!(!order.contains(&IssueFocus::MetaLabels), "{order:?}");
        assert!(!order.contains(&IssueFocus::Labels), "{order:?}");
        let area = Rect::new(0, 0, 100, 30);
        let mut buf = Buffer::empty(area);
        form.render(issue_rect(area), &mut buf);
        let text = crate::dialog_kit::buffer_text(&buf);
        assert!(text.contains("Assignees: none"), "{text}");
        assert!(text.contains("Milestone: none"), "{text}");
        assert!(!text.contains("Labels"), "{text}");
    }

    #[test]
    fn a_title_is_required_and_labels_split_on_commas() {
        let mut form = IssueDialog::new(trello_seed());
        assert_eq!(form.plan().unwrap_err(), "enter a title");
        assert_eq!(form.submit(), IssueOutcome::Stay);
        assert!(form.message.is_some());
        type_text(&mut form, "  It breaks ");
        form.labels = "bug, ,ui ".into();
        form.md.set_text("steps\n".into());
        let req = form.plan().unwrap();
        assert_eq!(req.project, "g/p");
        assert_eq!(req.title, "It breaks");
        assert_eq!(req.body, "steps");
        assert_eq!(req.labels, vec!["bug", "ui"]);
    }

    #[test]
    fn keyboard_walks_the_form_and_ctrl_s_or_the_button_submits() {
        let mut form = IssueDialog::new(seed());
        type_text(&mut form, "T");
        // Enter in Title moves to the description, where Enter is a newline.
        form.on_key(key(KeyCode::Enter));
        assert_eq!(form.focus, IssueFocus::Body);
        type_text(&mut form, "a");
        form.on_key(key(KeyCode::Enter));
        type_text(&mut form, "b");
        assert_eq!(form.md.text(), "a\nb");
        // GitLab: the metadata buttons replace the typed labels.
        for want in [
            IssueFocus::MetaAssignees,
            IssueFocus::MetaLabels,
            IssueFocus::MetaMilestone,
            IssueFocus::Web,
            IssueFocus::Create,
        ] {
            form.on_key(key(KeyCode::Tab));
            assert_eq!(form.focus, want);
        }
        form.meta.selection.apply(meta_picker::Applied::List(
            MetaField::Labels,
            vec!["bug".into()],
        ));
        match form.on_key(key(KeyCode::Enter)) {
            IssueOutcome::Submit { req, .. } => {
                assert_eq!(req.title, "T");
                assert_eq!(req.body, "a\nb");
                assert_eq!(req.labels, vec!["bug"]);
            }
            other => panic!("{other:?}"),
        }
        // Typed text: Esc asks first; Discard (second button) closes.
        assert_eq!(form.on_key(key(KeyCode::Esc)), IssueOutcome::Stay);
        form.on_key(key(KeyCode::Right));
        assert_eq!(form.on_key(key(KeyCode::Enter)), IssueOutcome::Cancel);
    }

    #[test]
    fn the_metadata_buttons_open_the_picker_and_the_choices_ride_along() {
        let mut form = IssueDialog::new(seed());
        form.fetch_options(Arc::new(crate::hosting_task::DefaultFactory), Vec::new());
        type_text(&mut form, "T");
        let area = Rect::new(0, 0, 100, 30);
        let mut buf = Buffer::empty(area);
        form.render(issue_rect(area), &mut buf);
        let text = crate::dialog_kit::buffer_text(&buf);
        for want in ["Assignees: none", "Labels: none", "Milestone: none"] {
            assert!(text.contains(want), "{want}\n{text}");
        }
        assert!(
            !text.contains("comma-separated"),
            "the typed labels field is gone on GitLab:\n{text}"
        );
        // A click on a button opens the picker over the form; Esc closes the
        // picker and leaves the form open.
        let labels = form.hit_rect(IssueFocus::MetaLabels);
        let click = MouseEvent {
            kind: MouseEventKind::Down(MouseButton::Left),
            column: labels.x + 1,
            row: labels.y,
            modifiers: KeyModifiers::NONE,
        };
        assert_eq!(form.on_mouse(click), IssueOutcome::Stay);
        assert!(form.meta.is_open());
        assert_eq!(form.on_key(key(KeyCode::Esc)), IssueOutcome::Stay);
        assert!(!form.meta.is_open());
        form.meta.selection.apply(meta_picker::Applied::List(
            MetaField::Assignees,
            vec!["ann".into()],
        ));
        form.meta
            .selection
            .apply(meta_picker::Applied::Milestone(Some(
                meta_picker::MilestoneRef {
                    id: 3,
                    title: "v1".into(),
                },
            )));
        let req = form.plan().unwrap();
        assert_eq!(req.assignees, ["ann"]);
        assert_eq!(req.milestone, Some(3));
    }

    #[test]
    fn the_web_button_and_key_open_the_prefilled_form_even_after_an_error() {
        let mut form = IssueDialog::new(seed());
        type_text(&mut form, "T x");
        form.message = Some("the host said no".into());
        let alt_w = KeyEvent::new(KeyCode::Char('w'), KeyModifiers::ALT);
        let want = "http://h/g/p/-/issues/new?issue%5Btitle%5D=T%20x";
        assert_eq!(form.on_key(alt_w), IssueOutcome::OpenWeb(want.to_string()));
        let area = Rect::new(0, 0, 100, 30);
        let mut buf = Buffer::empty(area);
        form.render(issue_rect(area), &mut buf);
        assert!(crate::dialog_kit::buffer_text(&buf).contains("Create on GitLab"));
        let web = form.hit_rect(IssueFocus::Web);
        let click = MouseEvent {
            kind: MouseEventKind::Down(MouseButton::Left),
            column: web.x + 1,
            row: web.y,
            modifiers: KeyModifiers::NONE,
        };
        assert_eq!(form.on_mouse(click), IssueOutcome::OpenWeb(want.into()));
    }

    #[test]
    fn a_busy_form_does_not_submit_twice() {
        let mut form = IssueDialog::new(seed());
        type_text(&mut form, "T");
        form.busy = true;
        assert_eq!(form.submit(), IssueOutcome::Stay);
    }

    fn click_xy(form: &mut IssueDialog, x: u16, y: u16) -> IssueOutcome {
        form.on_mouse(MouseEvent {
            kind: MouseEventKind::Down(MouseButton::Left),
            column: x,
            row: y,
            modifiers: KeyModifiers::NONE,
        })
    }

    fn painted(form: &mut IssueDialog) -> String {
        let area = Rect::new(0, 0, 100, 30);
        let mut buf = Buffer::empty(area);
        form.render(issue_rect(area), &mut buf);
        crate::dialog_kit::buffer_text(&buf)
    }

    #[test]
    fn the_cross_and_an_outside_click_close_a_pristine_form_without_asking() {
        let mut form = IssueDialog::new(seed());
        let text = painted(&mut form);
        assert!(text.contains('\u{2715}'), "{text}");
        assert!(!text.contains("Esc"), "no Esc hint\n{text}");
        assert_eq!(click_xy(&mut form, 0, 0), IssueOutcome::Cancel);
        let cross = form.guard.close.rect().expect("a cross");
        assert_eq!(
            click_xy(&mut form, cross.x + 1, cross.y),
            IssueOutcome::Cancel
        );
        assert_eq!(form.on_key(key(KeyCode::Esc)), IssueOutcome::Cancel);
    }

    #[test]
    fn a_typed_form_asks_on_esc_the_cross_and_outside_and_keeps_editing_on_cancel() {
        let mut form = IssueDialog::new(seed());
        type_text(&mut form, "T");
        painted(&mut form);
        let cross = form.guard.close.rect().expect("a cross");
        for name in ["esc", "cross", "outside"] {
            let out = match name {
                "esc" => form.on_key(key(KeyCode::Esc)),
                "cross" => click_xy(&mut form, cross.x + 1, cross.y),
                _ => click_xy(&mut form, 0, 0),
            };
            assert_eq!(out, IssueOutcome::Stay, "{name} asks first");
            let text = painted(&mut form);
            assert!(text.contains("Discard changes?"), "{name}\n{text}");
            // Esc answers Keep editing: the form is still open and typed.
            assert_eq!(form.on_key(key(KeyCode::Esc)), IssueOutcome::Stay);
            assert_eq!(form.title, "T");
            assert!(!painted(&mut form).contains("Discard changes?"));
        }
    }

    #[test]
    fn the_form_is_french_in_french() {
        let _l = gitpoulpi_i18n::override_lang(gitpoulpi_i18n::Lang::Fr);
        let mut form = IssueDialog::new(seed());
        assert_eq!(form.plan().unwrap_err(), "saisissez un titre");
        let area = Rect::new(0, 0, 100, 30);
        let mut buf = Buffer::empty(area);
        form.render(issue_rect(area), &mut buf);
        let text = crate::dialog_kit::buffer_text(&buf);
        for want in [
            "Nouveau ticket",
            "Titre",
            "Description",
            "Assignés: aucun",
            "Créer le ticket",
            "Annuler",
            "Tab suivant",
        ] {
            assert!(text.contains(want), "{want}\n{text}");
        }
    }

    #[test]
    fn the_buttons_are_hit_rects_from_the_paint_walk_and_click_through() {
        let mut form = IssueDialog::new(seed());
        type_text(&mut form, "T");
        let area = Rect::new(0, 0, 100, 30);
        let mut buf = Buffer::empty(area);
        form.render(issue_rect(area), &mut buf);
        let text: String = (0..area.height)
            .map(|y| {
                (0..area.width)
                    .map(|x| buf[(x, y)].symbol().to_string())
                    .collect::<String>()
            })
            .collect::<Vec<_>>()
            .join("\n");
        for want in [
            "New issue",
            "corp",
            "g/p",
            "Title",
            "Description",
            "Labels",
            "Create issue",
        ] {
            assert!(text.contains(want), "{want}:\n{text}");
        }
        let create = form.hit_rect(IssueFocus::Create);
        let click = |x, y| MouseEvent {
            kind: MouseEventKind::Down(MouseButton::Left),
            column: x,
            row: y,
            modifiers: KeyModifiers::NONE,
        };
        match form.on_mouse(click(create.x + 1, create.y)) {
            IssueOutcome::Submit { req, .. } => assert_eq!(req.title, "T"),
            other => panic!("{other:?}"),
        }
        let cancel = form.hit_rect(IssueFocus::Cancel);
        // Typed text: Cancel asks first, Keep editing leaves it open.
        assert_eq!(
            form.on_mouse(click(cancel.x + 1, cancel.y)),
            IssueOutcome::Stay
        );
        form.on_key(key(KeyCode::Right));
        assert_eq!(form.on_key(key(KeyCode::Enter)), IssueOutcome::Cancel);
    }

    #[test]
    fn a_jira_form_fetches_its_types_picks_the_default_and_sends_the_choice() {
        let transport = Arc::new(gitpoulpi_hosting::RecordedTransport::new().on(
            "/issue/createmeta/ABC/issuetypes",
            200,
            r#"{"values":[{"id":"10002","name":"Task"},{"id":"10004","name":"Bug"}],"total":2}"#,
        ));
        let jira = IssueSeed {
            integration: Integration::new("jira", IntegrationKind::JiraDataCenter, "http://j"),
            provider: "acme".into(),
            project: "ABC".into(),
        };
        let mut form = IssueDialog::new(jira);
        form.fetch_options(Arc::new(FakeFactory { transport }), vec![]);
        for _ in 0..200 {
            if form.poll() {
                break;
            }
            std::thread::sleep(std::time::Duration::from_millis(10));
        }
        // The default (Task) is chosen; Tab reaches the dropdown after Title.
        type_text(&mut form, "T");
        form.on_key(key(KeyCode::Tab));
        assert_eq!(form.focus, IssueFocus::Choice);
        assert_eq!(form.plan().unwrap().issue_type.as_deref(), Some("10002"));
        // Enter opens it, Down moves to Bug, Enter picks.
        form.on_key(key(KeyCode::Enter));
        form.on_key(key(KeyCode::Down));
        form.on_key(key(KeyCode::Enter));
        assert_eq!(form.plan().unwrap().issue_type.as_deref(), Some("10004"));
        let area = Rect::new(0, 0, 100, 30);
        let mut buf = Buffer::empty(area);
        form.render(issue_rect(area), &mut buf);
        let text = crate::dialog_kit::buffer_text(&buf);
        assert!(text.contains("Type") && text.contains("Bug"), "{text}");
        // GitLab has no such dropdown.
        assert!(IssueDialog::new(seed()).choice.is_none());
    }

    #[test]
    fn a_form_opened_from_a_board_column_starts_in_that_list_or_with_that_label() {
        // Trello: the column is the list, found by its name once the lists arrive.
        let transport = Arc::new(gitpoulpi_hosting::RecordedTransport::new().on(
            "/lists?filter=open",
            200,
            r#"[{"id":"l1","name":"To Do"},{"id":"l2","name":"Doing"}]"#,
        ));
        let mut form = IssueDialog::new(trello_seed());
        form.start_in(Some("doing".into()), None);
        form.fetch_options(Arc::new(FakeFactory { transport }), vec![]);
        for _ in 0..200 {
            if form.poll() {
                break;
            }
            std::thread::sleep(std::time::Duration::from_millis(10));
        }
        type_text(&mut form, "T");
        assert_eq!(form.plan().unwrap().list_id.as_deref(), Some("l2"));
        // A host board: the column's label rides along (typed labels here,
        // the picker's selection on GitHub / GitLab).
        let mut plain = IssueDialog::new(trello_seed());
        plain.start_in(None, Some("status::doing".into()));
        type_text(&mut plain, "T");
        assert_eq!(plain.plan().unwrap().labels, vec!["status::doing"]);
        let mut gitlab = IssueDialog::new(seed());
        gitlab.start_in(None, Some("status::doing".into()));
        type_text(&mut gitlab, "T");
        assert_eq!(gitlab.plan().unwrap().labels, vec!["status::doing"]);
    }

    struct FakeFactory {
        transport: Arc<gitpoulpi_hosting::RecordedTransport>,
    }

    impl ClientFactory for FakeFactory {
        fn client(
            &self,
            integration: &Integration,
            _all: &[Integration],
        ) -> gitpoulpi_hosting::Result<Box<dyn gitpoulpi_hosting::HostingClient>> {
            gitpoulpi_hosting::client_for(
                integration.clone(),
                "tok".into(),
                self.transport.clone(),
                &gitpoulpi_hosting::TrustGate::for_tests(),
            )
        }
    }

    fn wait(rx: &IssueReceiver) -> IssueEvent {
        for _ in 0..200 {
            if let Some(event) = rx.try_recv() {
                return event;
            }
            std::thread::sleep(std::time::Duration::from_millis(10));
        }
        panic!("the task never answered");
    }

    #[test]
    fn the_task_creates_through_the_client_and_reports_the_host_refusal() {
        let ok = Arc::new(gitpoulpi_hosting::RecordedTransport::new().on(
            "/issues",
            201,
            r#"{"iid":3,"web_url":"http://h/g/p/-/issues/3"}"#,
        ));
        let seed = seed();
        let req = NewIssue {
            project: "g/p".into(),
            title: "T".into(),
            ..Default::default()
        };
        let rx = start_create(
            Arc::new(FakeFactory {
                transport: ok.clone(),
            }),
            seed.integration.clone(),
            vec![],
            req.clone(),
        );
        let IssueEvent::Finished(Ok(created)) = wait(&rx) else {
            panic!("expected a created issue");
        };
        assert_eq!(created.number, 3);
        assert_eq!(ok.requests().len(), 1);

        let refused = Arc::new(gitpoulpi_hosting::RecordedTransport::new().on(
            "/issues",
            403,
            r#"{"message":"403 Forbidden"}"#,
        ));
        let rx = start_create(
            Arc::new(FakeFactory { transport: refused }),
            seed.integration,
            vec![],
            req,
        );
        let IssueEvent::Finished(Err(why)) = wait(&rx) else {
            panic!("expected an error");
        };
        assert!(why.contains("403") || why.contains("Forbidden"), "{why}");
    }
}
