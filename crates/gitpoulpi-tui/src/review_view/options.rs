//! The "✦ Review options" dialog: what one ✦ draft run is given besides the
//! diff (slice S5b of the radar-review design).
//!
//! `s` (button, palette) opens it prefilled with the defaults - the global
//! `[ai.review]` table with the repository's own values over it - and `Enter`
//! runs. Every choice made here is for that run only, unless "Save as this
//! repository's defaults" is ticked: then the values are also kept in the
//! repository's `repo-state.toml`, or in `remote-reviews.toml` for a
//! repository that is not cloned.
//!
//! The dialog owns its keys and its mouse while open; hit rects come out of
//! the walk that paints. It does not depend on the `ai` feature (the ✦
//! controls never show without it), so it only uses config and session types.

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers, MouseButton, MouseEvent, MouseEventKind};
use gitpoulpi_config::{AiReviewContext, RepoAiReview, RunInRepo};
use gitpoulpi_i18n::msg::review_options as msg;
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::Style;

use crate::combo::{Combo, ComboOption, ComboOutcome};
use crate::dialog_kit::{self, ButtonState, CheckState, CloseHit, Dismiss, DismissAction};
use crate::file_browse::{BrowseOutcome, FileBrowse};
use crate::keymap::Action as Bind;
use crate::text_edit::FieldEditor;
use crate::th;

/// What the dialog starts from, and what it needs to know about the run.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(super) struct Defaults {
    pub(super) global: AiReviewContext,
    /// The repository's (or the project's) own values.
    pub(super) own: RepoAiReview,
    /// Why "run the agent in the repository" cannot be chosen (an HTTP
    /// profile, or no clone); `None`: it can.
    pub(super) run_blocked: Option<String>,
    /// The profile's own working folder is the repository: with "Profile
    /// setting" the agent reads the files by itself.
    pub(super) profile_in_repository: bool,
    /// The folder name of a clone's state file (`session.per_repo_state_dir`).
    pub(super) state_dir: String,
    /// The clone's root: where Browse starts and what a chosen file is
    /// stored relative to. `None` without a clone.
    pub(super) repo_root: Option<std::path::PathBuf>,
}

impl Defaults {
    /// The values the dialog shows: the repository's over the global ones.
    /// (`gitpoulpi_ai::review_context::resolve` does the same; this copy
    /// keeps the dialog independent of the `ai` feature.)
    pub(super) fn shown(&self) -> AiReviewContext {
        AiReviewContext {
            run_in_repository: self
                .own
                .run_in_repository
                .unwrap_or(self.global.run_in_repository),
            include_claude_md: self
                .own
                .include_claude_md
                .unwrap_or(self.global.include_claude_md),
            include_agents_md: self
                .own
                .include_agents_md
                .unwrap_or(self.global.include_agents_md),
            extra_file: self
                .own
                .extra_choice()
                .unwrap_or_else(|| self.global.extra_file.clone())
                .trim()
                .to_string(),
        }
    }
}

/// Reads the defaults for the review of a request: the global `[ai.review]`
/// table, the repository's own values (a clone's state file, or the
/// project's entry of `remote-reviews.toml`) and what the AI profile of the
/// code review allows. Small files, read when the dialog opens.
pub(super) fn load_defaults(
    repo_root: Option<&std::path::Path>,
    remote_file: Option<&std::path::Path>,
    remote_key: &str,
) -> Defaults {
    use gitpoulpi_config::{AgentWorkdir, AiAction, AiProviderKind};
    use gitpoulpi_session::{RemoteReviews, RepoState};
    let Ok(settings) = gitpoulpi_config::Settings::resolve_ambient() else {
        return Defaults::default();
    };
    let config = settings.config();
    let ai = &config.ai;
    let state_dir = config.session.per_repo_state_dir.clone();
    let state = repo_root
        .map(|root| RepoState::load(&RepoState::path_for(root, &state_dir)).unwrap_or_default());
    let own = match (&state, remote_file) {
        (Some(state), _) => state.preferences.ai_review.clone(),
        (None, Some(file)) => RemoteReviews::load(file)
            .ai_review
            .get(remote_key)
            .cloned()
            .unwrap_or_default(),
        (None, None) => RepoAiReview::default(),
    };
    // The profile the review will use: the repository's choice, else the
    // action's, else the default one.
    let from_repo = state
        .as_ref()
        .and_then(|s| s.preferences.ai_override(AiAction::CodeReview.key(), ai))
        .map(|o| o.profile)
        .filter(|id| !id.is_empty());
    let id = from_repo
        .or_else(|| Some(ai.actions.get(AiAction::CodeReview).profile.clone()))
        .filter(|id| !id.is_empty())
        .unwrap_or_else(|| ai.default_profile.clone());
    let profile = ai
        .profile(&id)
        .or_else(|| ai.profile(&ai.default_profile))
        .or_else(|| ai.profiles.first());
    let local = profile.is_some_and(|p| p.kind == AiProviderKind::LocalAgent);
    let run_blocked = if repo_root.is_none() {
        Some(msg::WHY_API.get().to_string())
    } else if profile.is_some() && !local {
        Some(msg::WHY_HTTP.get().to_string())
    } else {
        None
    };
    Defaults {
        global: ai.review.clone(),
        own,
        run_blocked,
        profile_in_repository: local
            && profile.is_some_and(|p| p.workdir == AgentWorkdir::Repository),
        state_dir,
        repo_root: repo_root.map(std::path::Path::to_path_buf),
    }
}

/// Keeps `values` as the repository's own review defaults: in a clone's
/// state file, else in the project's entry of `remote-reviews.toml`.
pub(super) fn save_defaults(
    repo_root: Option<&std::path::Path>,
    remote_file: Option<&std::path::Path>,
    remote_key: &str,
    state_dir: &str,
    values: &RepoAiReview,
) -> Result<(), String> {
    use gitpoulpi_session::{RemoteReviews, RepoState};
    match (repo_root, remote_file) {
        (Some(root), _) => {
            let path = RepoState::path_for(root, state_dir);
            let mut state = RepoState::load(&path).map_err(|e| e.to_string())?;
            state.preferences.ai_review = values.clone();
            state.save(&path).map_err(|e| e.to_string())
        }
        (None, Some(file)) => {
            let mut all = RemoteReviews::load(file);
            if values.is_empty() {
                all.ai_review.remove(remote_key);
            } else {
                all.ai_review.insert(remote_key.to_string(), values.clone());
            }
            all.save(file).map_err(|e| e.to_string())
        }
        (None, None) => Err(msg::NO_PLACE.get().to_string()),
    }
}

/// What the reviewer decided.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct Choice {
    /// The values of this run (every field set; the run choice is left
    /// unset when it cannot be made).
    pub(super) per_run: RepoAiReview,
    /// Also keep them as the repository's defaults.
    pub(super) save: Option<RepoAiReview>,
    /// Where a clone's state file lives (`session.per_repo_state_dir`).
    pub(super) state_dir: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) enum OptionsOutcome {
    Stay,
    Cancel,
    Run(Choice),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Part {
    Run,
    Claude,
    Agents,
    Extra,
    /// The "Browse..." button of the extra file field.
    Browse,
    /// The "No extra file" checkbox.
    NoExtra,
    Save,
    Go,
    Cancel,
}

pub(super) struct OptionsDialog {
    defaults: Defaults,
    run: Combo,
    claude: bool,
    agents: bool,
    extra: String,
    /// "No extra file": the run gets none, whatever the field and the
    /// global setting say. The field keeps its text meanwhile.
    no_extra: bool,
    save: bool,
    /// What the dialog opened with, to tell whether the reviewer changed
    /// anything.
    pristine: (String, bool, bool, String, bool),
    /// The file picker, while "Browse..." is open over the dialog.
    browse: Option<FileBrowse>,
    edit: FieldEditor<Part>,
    focus: Part,
    hover: Option<Part>,
    hits: Vec<(Rect, Part)>,
    close: CloseHit,
    panel: Rect,
}

fn run_key(run: RunInRepo) -> &'static str {
    run.key()
}

impl OptionsDialog {
    pub(super) fn new(defaults: Defaults) -> Self {
        let shown = defaults.shown();
        let mut run = Combo::new(msg::RUN_PROFILE.get());
        run.set_options(vec![
            ComboOption::new(msg::RUN_PROFILE.get(), run_key(RunInRepo::Profile)),
            ComboOption::new(msg::RUN_YES.get(), run_key(RunInRepo::Yes)),
            ComboOption::new(msg::RUN_NO.get(), run_key(RunInRepo::No)),
        ]);
        run.select(Some(run_key(shown.run_in_repository).to_string()));
        let focus = if defaults.run_blocked.is_some() {
            Part::Claude
        } else {
            Part::Run
        };
        let no_extra = defaults.own.extra_choice().is_some_and(|f| f.is_empty());
        let pristine = (
            run_key(shown.run_in_repository).to_string(),
            shown.include_claude_md,
            shown.include_agents_md,
            shown.extra_file.clone(),
            no_extra,
        );
        let mut edit = FieldEditor::default();
        edit.caret_mut(Part::Extra);
        Self {
            run,
            claude: shown.include_claude_md,
            agents: shown.include_agents_md,
            extra: shown.extra_file,
            no_extra,
            save: false,
            pristine,
            browse: None,
            edit,
            focus,
            hover: None,
            hits: Vec::new(),
            close: CloseHit::default(),
            panel: Rect::default(),
            defaults,
        }
    }

    /// The run choice as the combobox holds it.
    fn run_choice(&self) -> RunInRepo {
        self.run
            .selected()
            .and_then(RunInRepo::from_key)
            .unwrap_or_default()
    }

    /// With this choice the agent works inside the repository, so it reads
    /// `CLAUDE.md` and `AGENTS.md` on its own.
    fn agent_in_repository(&self) -> bool {
        self.defaults.run_blocked.is_none()
            && match self.run_choice() {
                RunInRepo::Yes => true,
                RunInRepo::No => false,
                RunInRepo::Profile => self.defaults.profile_in_repository,
            }
    }

    /// The values as the dialog holds them now.
    pub(super) fn values(&self) -> AiReviewContext {
        AiReviewContext {
            run_in_repository: self.run_choice(),
            include_claude_md: self.claude,
            include_agents_md: self.agents,
            extra_file: if self.no_extra {
                String::new()
            } else {
                self.extra.trim().to_string()
            },
        }
    }

    /// The extra file field, as the repository or the global setting would
    /// give it: what comes back when "No extra file" is unticked on an empty
    /// field.
    fn inherited_extra(&self) -> String {
        self.defaults
            .own
            .extra_file
            .clone()
            .map(|f| f.trim().to_string())
            .filter(|f| !f.is_empty())
            .unwrap_or_else(|| self.defaults.global.extra_file.trim().to_string())
    }

    /// Unticks "No extra file". `refill`: an empty field gets its inherited
    /// text back (the checkbox was clicked); typing into the field does not
    /// want it.
    fn untick_no_extra(&mut self, refill: bool) {
        if !self.no_extra {
            return;
        }
        self.no_extra = false;
        if refill && self.extra.trim().is_empty() {
            self.extra = self.inherited_extra();
            let end = self.extra.len();
            self.edit
                .caret_mut(Part::Extra)
                .set_cursor(&self.extra, end, false);
        }
    }

    /// Opens the file picker over the dialog.
    fn open_browse(&mut self) {
        self.focus = Part::Browse;
        self.browse = Some(FileBrowse::open(
            msg::BROWSE_TITLE.get(),
            msg::BROWSE_CONFIRM.get(),
            self.defaults.repo_root.as_deref(),
            self.extra.trim(),
        ));
    }

    /// The picker answered.
    fn browsed(&mut self, outcome: BrowseOutcome) {
        match outcome {
            BrowseOutcome::Stay => {}
            BrowseOutcome::Cancel => self.browse = None,
            BrowseOutcome::Chosen(path) => {
                self.browse = None;
                self.no_extra = false;
                self.extra = path;
                let end = self.extra.len();
                self.edit
                    .caret_mut(Part::Extra)
                    .set_cursor(&self.extra, end, false);
                self.focus = Part::Extra;
            }
        }
    }

    fn outcome(&self) -> OptionsOutcome {
        let values = self.values();
        let blocked = self.defaults.run_blocked.is_some();
        // An empty field is "none" here: the dialog starts from what the
        // run would inherit, so nothing is left to inherit.
        let none = values.extra_file.is_empty();
        let per_run = RepoAiReview {
            run_in_repository: (!blocked).then_some(values.run_in_repository),
            include_claude_md: Some(values.include_claude_md),
            include_agents_md: Some(values.include_agents_md),
            extra_file: (!none).then(|| values.extra_file.clone()),
            no_extra_file: none.then_some(true),
        };
        let save = self.save.then(|| {
            // A field is pinned for the repository when it differs from the
            // global value or the repository already had its own: a value
            // that merely agrees with the global one keeps following it.
            let (global, own) = (&self.defaults.global, &self.defaults.own);
            let (extra_file, no_extra_file) =
                if values.extra_file == global.extra_file.trim() && own.extra_choice().is_none() {
                    (None, None)
                } else if none {
                    (None, Some(true))
                } else {
                    (Some(values.extra_file.clone()), None)
                };
            RepoAiReview {
                run_in_repository: if blocked {
                    own.run_in_repository
                } else {
                    pin(
                        values.run_in_repository,
                        global.run_in_repository,
                        own.run_in_repository,
                    )
                },
                include_claude_md: pin(
                    values.include_claude_md,
                    global.include_claude_md,
                    own.include_claude_md,
                ),
                include_agents_md: pin(
                    values.include_agents_md,
                    global.include_agents_md,
                    own.include_agents_md,
                ),
                extra_file,
                no_extra_file,
            }
        });
        OptionsOutcome::Run(Choice {
            per_run,
            save,
            state_dir: self.defaults.state_dir.clone(),
        })
    }

    fn order(&self) -> Vec<Part> {
        let mut order = Vec::new();
        if self.defaults.run_blocked.is_none() {
            order.push(Part::Run);
        }
        order.extend([
            Part::Claude,
            Part::Agents,
            Part::Extra,
            Part::Browse,
            Part::NoExtra,
            Part::Save,
            Part::Go,
            Part::Cancel,
        ]);
        order
    }

    fn step(&mut self, forward: bool) {
        let order = self.order();
        let at = order.iter().position(|p| *p == self.focus).unwrap_or(0);
        let n = order.len();
        self.focus = order[if forward {
            (at + 1) % n
        } else {
            (at + n - 1) % n
        }];
    }

    fn toggle(&mut self, part: Part) {
        match part {
            Part::Claude => self.claude = !self.claude,
            Part::Agents => self.agents = !self.agents,
            Part::Save => self.save = !self.save,
            Part::NoExtra if self.no_extra => self.untick_no_extra(true),
            Part::NoExtra => self.no_extra = true,
            _ => {}
        }
    }

    pub(super) fn paste(&mut self, text: &str) {
        if let Some(browse) = self.browse.as_mut() {
            browse.paste(text);
            return;
        }
        if self.focus == Part::Extra {
            self.untick_no_extra(false);
            let clean: String = text.chars().filter(|c| !c.is_control()).collect();
            self.edit.sync(Some(Part::Extra));
            let mut value = std::mem::take(&mut self.extra);
            self.edit.caret_mut(Part::Extra).insert(&mut value, &clean);
            self.extra = value;
        }
    }

    pub(super) fn on_key(&mut self, key: KeyEvent) -> OptionsOutcome {
        if let Some(browse) = self.browse.as_mut() {
            let outcome = browse.on_key(key);
            self.browsed(outcome);
            return OptionsOutcome::Stay;
        }
        if self.run.is_open() {
            self.run.on_key(key);
            return OptionsOutcome::Stay;
        }
        if crate::keymap::current().is(Bind::DialogBrowse, &key) {
            self.open_browse();
            return OptionsOutcome::Stay;
        }
        self.edit
            .sync((self.focus == Part::Extra).then_some(Part::Extra));
        if self.focus == Part::Extra {
            let edits = matches!(key.code, KeyCode::Backspace | KeyCode::Delete)
                || matches!(key.code, KeyCode::Char(_))
                    && !key
                        .modifiers
                        .intersects(KeyModifiers::CONTROL | KeyModifiers::ALT);
            if edits {
                self.untick_no_extra(false);
            }
            let mut value = std::mem::take(&mut self.extra);
            let handled = self.edit.dialog_key(Part::Extra, &mut value, &key);
            self.extra = value;
            if handled.is_some() {
                return OptionsOutcome::Stay;
            }
        }
        let shift = key.modifiers.contains(KeyModifiers::SHIFT);
        match key.code {
            KeyCode::Esc => return OptionsOutcome::Cancel,
            KeyCode::Tab | KeyCode::Down => self.step(!shift),
            KeyCode::BackTab | KeyCode::Up => self.step(false),
            _ if crate::keymap::current().is(Bind::DialogSubmit, &key) => return self.outcome(),
            KeyCode::Enter => {
                return match self.focus {
                    Part::Cancel => OptionsOutcome::Cancel,
                    Part::Browse => {
                        self.open_browse();
                        OptionsOutcome::Stay
                    }
                    _ => self.outcome(),
                };
            }
            KeyCode::Char(' ') => match self.focus {
                Part::Claude | Part::Agents | Part::Save | Part::NoExtra => self.toggle(self.focus),
                Part::Browse => self.open_browse(),
                Part::Run => {
                    self.run.on_key(key);
                }
                Part::Go => return self.outcome(),
                Part::Cancel => return OptionsOutcome::Cancel,
                Part::Extra => {}
            },
            _ => {
                if self.focus == Part::Run && self.run.on_key(key) == ComboOutcome::Handled {
                    return OptionsOutcome::Stay;
                }
            }
        }
        OptionsOutcome::Stay
    }

    pub(super) fn on_mouse(&mut self, mouse: MouseEvent) -> OptionsOutcome {
        if let Some(browse) = self.browse.as_mut() {
            let outcome = browse.on_mouse(mouse);
            self.browsed(outcome);
            return OptionsOutcome::Stay;
        }
        if self.run.is_open() {
            self.run.on_mouse(mouse);
            return OptionsOutcome::Stay;
        }
        if !matches!(mouse.kind, MouseEventKind::Down(_))
            && self.edit.dialog_mouse(
                Some(Part::Extra),
                &self.extra,
                mouse.kind,
                mouse.column,
                mouse.row,
            )
        {
            return OptionsOutcome::Stay;
        }
        self.close.set_hover(mouse.column, mouse.row);
        let hit = dialog_kit::hit_at(&self.hits, mouse.column, mouse.row);
        match mouse.kind {
            MouseEventKind::Moved => self.hover = hit,
            MouseEventKind::Down(MouseButton::Left) => {
                if self.close.hit(mouse.column, mouse.row) {
                    return OptionsOutcome::Cancel;
                }
                match hit {
                    Some(Part::Run) => {
                        self.focus = Part::Run;
                        self.run.on_mouse(mouse);
                    }
                    Some(part @ (Part::Claude | Part::Agents | Part::Save | Part::NoExtra)) => {
                        self.focus = part;
                        self.toggle(part);
                    }
                    Some(Part::Browse) => self.open_browse(),
                    Some(Part::Extra) => {
                        self.focus = Part::Extra;
                        self.untick_no_extra(false);
                        self.edit.sync(Some(Part::Extra));
                        self.edit.dialog_mouse(
                            Some(Part::Extra),
                            &self.extra,
                            mouse.kind,
                            mouse.column,
                            mouse.row,
                        );
                    }
                    Some(Part::Go) => return self.outcome(),
                    Some(Part::Cancel) => return OptionsOutcome::Cancel,
                    None => {
                        // A click outside the box closes it, unless the
                        // reviewer changed something: then the cross blinks.
                        let outside = !self.panel.contains((mouse.column, mouse.row).into());
                        if outside {
                            match dialog_kit::dismiss(self, dialog_kit::DismissKind::Outside) {
                                DismissAction::Close => return OptionsOutcome::Cancel,
                                _ => self.close.request_flash(),
                            }
                        }
                    }
                }
            }
            MouseEventKind::Up(MouseButton::Left) => {
                self.edit.dialog_mouse(
                    Some(Part::Extra),
                    &self.extra,
                    mouse.kind,
                    mouse.column,
                    mouse.row,
                );
            }
            _ => {}
        }
        OptionsOutcome::Stay
    }

    pub(super) fn render(&mut self, area: Rect, buf: &mut Buffer) {
        let rect = dialog_kit::centered(area, area.width.saturating_sub(4).min(86), 15);
        self.panel = rect;
        self.edit.begin_frame();
        let title = format!("{} {}", crate::theme::AI_SPARK, msg::TITLE.get());
        self.close.panel(buf, rect, &title);
        let inner = Rect::new(
            rect.x + 2,
            rect.y + 1,
            rect.width.saturating_sub(4),
            rect.height.saturating_sub(2),
        );
        let mut hits: Vec<(Rect, Part)> = Vec::new();
        if inner.width < 30 || inner.height < 12 {
            self.hits = hits;
            return;
        }
        let muted = Style::default().fg(th().ui.muted).bg(th().ui.panel);
        let mut y = inner.y;

        // Where the agent runs.
        let label_w = (crate::display_width(msg::RUN_LABEL.get()) as u16 + 2).min(inner.width / 2);
        dialog_kit::put(buf, inner.x, y, msg::RUN_LABEL.get(), label_w, muted);
        let field = Rect::new(
            inner.x + label_w,
            y,
            inner.width.saturating_sub(label_w).min(30),
            1,
        );
        if let Some(why) = self.defaults.run_blocked.clone() {
            dialog_kit::fill(buf, field, Style::default().bg(th().ui.panel));
            dialog_kit::put(
                buf,
                field.x + 1,
                y,
                self.run.selected_label().unwrap_or_default(),
                field.width.saturating_sub(1),
                Style::default().fg(th().ui.disabled).bg(th().ui.panel),
            );
            y += 1;
            let lines = crate::ui_kit::wrap_words(&why, usize::from(inner.width));
            if let Some(line) = lines.first() {
                dialog_kit::put(
                    buf,
                    inner.x,
                    y,
                    line,
                    inner.width,
                    Style::default().fg(th().status.warning).bg(th().ui.panel),
                );
            }
        } else {
            self.run.render_field(
                buf,
                field,
                self.focus == Part::Run,
                self.hover == Some(Part::Run),
            );
            hits.push((field, Part::Run));
        }
        y += 2;

        // The two convention files.
        let note = format!("({})", msg::READS_ITSELF.get());
        let in_repo = self.agent_in_repository();
        for (part, label, on) in [
            (Part::Claude, "CLAUDE.md", self.claude), // i18n-english
            (Part::Agents, "AGENTS.md", self.agents), // i18n-english
        ] {
            let rect = dialog_kit::checkbox(
                buf,
                inner.x,
                y,
                inner.width,
                label,
                on,
                CheckState::Enabled {
                    hovered: self.hover == Some(part),
                    focused: self.focus == part,
                },
            );
            hits.push((rect, part));
            if in_repo {
                let x = rect.right() + 2;
                dialog_kit::put(buf, x, y, &note, inner.right().saturating_sub(x), muted);
            }
            y += 1;
        }
        y += 1;

        // One more file, by path or by link.
        dialog_kit::put(buf, inner.x, y, msg::EXTRA_LABEL.get(), inner.width, muted);
        y += 1;
        let browse_label = gitpoulpi_i18n::msg::dialog::BROWSE.get();
        let browse_w = dialog_kit::button_width(browse_label);
        let extra = Rect::new(inner.x, y, inner.width.saturating_sub(browse_w + 1), 1);
        let (shown, hint) = if self.no_extra {
            ("", msg::EXTRA_NONE.get())
        } else {
            (self.extra.as_str(), msg::EXTRA_PLACEHOLDER.get())
        };
        dialog_kit::text_input(
            buf,
            extra,
            shown,
            hint,
            self.focus == Part::Extra,
            self.hover == Some(Part::Extra),
            (&self.edit, Part::Extra),
        );
        hits.push((extra, Part::Extra));
        let browse = dialog_kit::button(
            buf,
            extra.right() + 1,
            y,
            browse_label,
            ButtonState::Normal {
                hovered: self.hover == Some(Part::Browse),
                focused: self.focus == Part::Browse,
            },
        );
        hits.push((browse, Part::Browse));
        y += 1;
        let no_extra = dialog_kit::checkbox(
            buf,
            inner.x,
            y,
            inner.width,
            msg::NO_EXTRA_LABEL.get(),
            self.no_extra,
            CheckState::Enabled {
                hovered: self.hover == Some(Part::NoExtra),
                focused: self.focus == Part::NoExtra,
            },
        );
        hits.push((no_extra, Part::NoExtra));
        y += 2;

        let save = dialog_kit::checkbox(
            buf,
            inner.x,
            y,
            inner.width,
            msg::SAVE_DEFAULTS.get(),
            self.save,
            CheckState::Enabled {
                hovered: self.hover == Some(Part::Save),
                focused: self.focus == Part::Save,
            },
        );
        hits.push((save, Part::Save));

        // Buttons, right to left, and the key hints.
        let button_y = inner.bottom().saturating_sub(1);
        let go_label = msg::RUN.get();
        let cancel_label = gitpoulpi_i18n::msg::review_view::CANCEL.get();
        let go_w = dialog_kit::button_width(go_label);
        let cancel_w = dialog_kit::button_width(cancel_label);
        let go_x = inner.right().saturating_sub(go_w);
        let cancel_x = go_x.saturating_sub(cancel_w + 1);
        let go = dialog_kit::button(
            buf,
            go_x,
            button_y,
            go_label,
            ButtonState::Primary {
                hovered: self.hover == Some(Part::Go),
                focused: self.focus == Part::Go,
            },
        );
        let cancel = dialog_kit::button(
            buf,
            cancel_x,
            button_y,
            cancel_label,
            ButtonState::Normal {
                hovered: self.hover == Some(Part::Cancel),
                focused: self.focus == Part::Cancel,
            },
        );
        hits.insert(0, (cancel, Part::Cancel));
        hits.insert(0, (go, Part::Go));
        let tab = dialog_kit::key_name(crate::keymap::chord::Key::Tab);
        let enter = dialog_kit::key_name(crate::keymap::chord::Key::Enter);
        let browse_key = dialog_kit::key_label(Bind::DialogBrowse);
        dialog_kit::hints(
            buf,
            inner.x,
            button_y,
            cancel_x.saturating_sub(inner.x + 1),
            &[
                (tab.as_str(), gitpoulpi_i18n::msg::dialog::HINT_NEXT.get()),
                (enter.as_str(), msg::HINT_RUN.get()),
                (
                    browse_key.as_str(),
                    gitpoulpi_i18n::msg::dialog::HINT_BROWSE.get(),
                ),
            ],
        );
        self.hits = hits;
        self.run.render_popup(buf, buf.area);
        if let Some(browse) = self.browse.as_mut() {
            browse.render(area, buf);
        }
    }

    /// The control rects of the last paint (tests).
    #[cfg(test)]
    pub(super) fn hit_of(&self, part: &str) -> Option<Rect> {
        let want = match part {
            "run" => Part::Run,
            "claude" => Part::Claude,
            "agents" => Part::Agents,
            "extra" => Part::Extra,
            "browse" => Part::Browse,
            "no_extra" => Part::NoExtra,
            "save" => Part::Save,
            "go" => Part::Go,
            _ => Part::Cancel,
        };
        self.hits.iter().find(|(_, p)| *p == want).map(|(r, _)| *r)
    }

    /// The cross of the last paint (tests).
    #[cfg(test)]
    pub(super) fn cross(&self) -> Option<Rect> {
        self.close.rect()
    }
}

/// `value` as the repository's own entry: set when it differs from the
/// global one or was already set.
fn pin<T: PartialEq>(value: T, global: T, own: Option<T>) -> Option<T> {
    if value != global || own.is_some() {
        Some(value)
    } else {
        None
    }
}

impl Dismiss for OptionsDialog {
    fn is_dirty(&self) -> bool {
        self.pristine
            != (
                run_key(self.run_choice()).to_string(),
                self.claude,
                self.agents,
                self.extra.trim().to_string(),
                self.no_extra,
            )
            || self.save
    }

    fn close_hit(&self) -> &CloseHit {
        &self.close
    }

    fn close_hit_mut(&mut self) -> &mut CloseHit {
        &mut self.close
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn defaults() -> Defaults {
        Defaults {
            global: AiReviewContext::default(),
            own: RepoAiReview::default(),
            run_blocked: None,
            profile_in_repository: false,
            state_dir: String::new(),
            repo_root: None,
        }
    }

    #[test]
    fn the_repository_values_win_over_the_global_ones() {
        let d = Defaults {
            global: AiReviewContext {
                include_agents_md: true,
                extra_file: "global.md".into(),
                ..Default::default()
            },
            own: RepoAiReview {
                include_agents_md: Some(false),
                run_in_repository: Some(RunInRepo::Yes),
                ..Default::default()
            },
            ..defaults()
        };
        let shown = d.shown();
        assert!(!shown.include_agents_md, "the repository's value");
        assert!(shown.include_claude_md, "unset inherits the global one");
        assert_eq!(shown.extra_file, "global.md");
        assert_eq!(shown.run_in_repository, RunInRepo::Yes);
        let dialog = OptionsDialog::new(d);
        assert_eq!(dialog.values(), shown, "the dialog starts from them");
    }

    #[test]
    fn a_value_that_follows_the_global_one_is_not_pinned_when_saved() {
        let mut dialog = OptionsDialog::new(defaults());
        dialog.agents = false;
        dialog.save = true;
        let OptionsOutcome::Run(choice) = dialog.outcome() else {
            panic!("runs")
        };
        let saved = choice.save.expect("saved");
        assert_eq!(saved.include_agents_md, Some(false), "differs: pinned");
        assert_eq!(saved.include_claude_md, None, "agrees: keeps following");
        assert_eq!(saved.run_in_repository, None);
        assert_eq!(saved.extra_file, None);
        assert_eq!(choice.per_run.include_agents_md, Some(false));
        assert_eq!(choice.per_run.include_claude_md, Some(true));
    }

    fn with_global_file() -> Defaults {
        Defaults {
            global: AiReviewContext {
                extra_file: "global.md".into(),
                ..Default::default()
            },
            ..defaults()
        }
    }

    #[test]
    fn no_extra_file_turns_the_global_file_off_for_the_run_and_the_saved_defaults() {
        let mut dialog = OptionsDialog::new(with_global_file());
        assert_eq!(dialog.values().extra_file, "global.md");
        dialog.toggle(Part::NoExtra);
        dialog.save = true;
        assert_eq!(dialog.values().extra_file, "", "the field's text is kept");
        let OptionsOutcome::Run(choice) = dialog.outcome() else {
            panic!("runs")
        };
        assert_eq!(choice.per_run.no_extra_file, Some(true));
        assert_eq!(choice.per_run.extra_file, None);
        let saved = choice.save.unwrap();
        assert_eq!(saved.no_extra_file, Some(true));
        assert_eq!(saved.extra_file, None);
        // Unticked again: the global file is back, and nothing is pinned.
        dialog.toggle(Part::NoExtra);
        let OptionsOutcome::Run(choice) = dialog.outcome() else {
            panic!("runs")
        };
        assert_eq!(choice.per_run.extra_file.as_deref(), Some("global.md"));
        assert_eq!(choice.save.unwrap().extra_file, None);
    }

    #[test]
    fn a_repository_that_has_no_extra_file_opens_ticked_and_unticking_restores_the_global_one() {
        let mut dialog = OptionsDialog::new(Defaults {
            own: RepoAiReview {
                no_extra_file: Some(true),
                ..Default::default()
            },
            ..with_global_file()
        });
        assert!(dialog.no_extra);
        assert_eq!(dialog.values().extra_file, "");
        assert!(!dialog.is_dirty(), "as opened");
        dialog.toggle(Part::NoExtra);
        assert_eq!(dialog.values().extra_file, "global.md");
        assert!(dialog.is_dirty());
    }

    #[test]
    fn an_empty_extra_field_is_none_for_the_run_and_typing_unticks_the_switch() {
        let mut dialog = OptionsDialog::new(with_global_file());
        dialog.extra.clear();
        let OptionsOutcome::Run(choice) = dialog.outcome() else {
            panic!("runs")
        };
        assert_eq!(choice.per_run.no_extra_file, Some(true));
        dialog.toggle(Part::NoExtra);
        dialog.focus = Part::Extra;
        dialog.on_key(KeyEvent::new(KeyCode::Char('a'), KeyModifiers::NONE));
        assert!(!dialog.no_extra);
        assert_eq!(dialog.values().extra_file, "a");
    }

    #[test]
    fn browse_is_in_the_tab_order_and_a_chosen_file_fills_the_field() {
        let mut dialog = OptionsDialog::new(with_global_file());
        assert!(dialog.order().contains(&Part::Browse));
        dialog.toggle(Part::NoExtra);
        dialog.browsed(BrowseOutcome::Chosen("docs/R.md".into()));
        assert!(!dialog.no_extra, "a file replaces the switch");
        assert_eq!(dialog.values().extra_file, "docs/R.md");
        assert_eq!(dialog.focus, Part::Extra);
    }

    #[test]
    fn ctrl_b_opens_the_picker_from_anywhere_and_escape_closes_only_the_picker() {
        let mut dialog = OptionsDialog::new(defaults());
        dialog.focus = Part::Claude;
        let ctrl_b = KeyEvent::new(KeyCode::Char('b'), KeyModifiers::CONTROL);
        assert_eq!(dialog.on_key(ctrl_b), OptionsOutcome::Stay);
        assert!(dialog.browse.is_some());
        let esc = KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE);
        assert_eq!(dialog.on_key(esc), OptionsOutcome::Stay, "not the dialog");
        assert!(dialog.browse.is_none());
        // Enter on the button opens it too.
        dialog.focus = Part::Browse;
        dialog.on_key(KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE));
        assert!(dialog.browse.is_some());
    }

    #[test]
    fn a_blocked_run_choice_is_left_out() {
        let mut dialog = OptionsDialog::new(Defaults {
            run_blocked: Some("no".into()),
            own: RepoAiReview {
                run_in_repository: Some(RunInRepo::Yes),
                ..Default::default()
            },
            ..defaults()
        });
        dialog.save = true;
        let OptionsOutcome::Run(choice) = dialog.outcome() else {
            panic!("runs")
        };
        assert_eq!(choice.per_run.run_in_repository, None);
        assert_eq!(
            choice.save.unwrap().run_in_repository,
            Some(RunInRepo::Yes),
            "the saved choice is not touched"
        );
        assert!(!dialog.order().contains(&Part::Run), "not focusable");
    }
}
