//! The Radar block of a connected code-host card: which account-wide lists
//! the Radar and `hosting radar` fetch for the profile
//! (`[integrations.radar]`), and what a request of an uncloned repository
//! does. Everything is saved at once, like the rest of the page.
//!
//! Hosts without account-wide lists (Bitbucket Cloud has no such query) get
//! one muted line saying so instead of controls that would do nothing: a
//! reason is clearer than a missing block.

use super::*;
use crate::combo::{Combo, ComboOption, ComboOutcome};
use crate::th;
use gitpoulpi_config::{ApiFamily, IntegrationRadar, RemoteReviewMode};
use gitpoulpi_i18n::msg::radar_settings as rmsg;

/// Whether the host can answer account-wide lists at all.
pub(super) fn has_lists(kind: IntegrationKind) -> bool {
    kind.category() == IntegrationCategory::Hosting && kind != IntegrationKind::Bitbucket
}

/// Whether the host has issues to list (Bitbucket and Gerrit have none).
fn has_issues(kind: IntegrationKind) -> bool {
    !matches!(kind.family(), ApiFamily::Bitbucket | ApiFamily::Gerrit)
}

/// The names typed in the Namespaces field: comma separated, trimmed,
/// empty ones and repeats dropped.
pub(super) fn parse_namespaces(text: &str) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    for name in text.split(',').map(str::trim).filter(|n| !n.is_empty()) {
        if !out.iter().any(|o| o == name) {
            out.push(name.to_string());
        }
    }
    out
}

fn mode_label(mode: RemoteReviewMode) -> &'static str {
    match mode {
        RemoteReviewMode::Ask => rmsg::MODE_ASK.get(),
        RemoteReviewMode::Clone => rmsg::MODE_CLONE.get(),
        RemoteReviewMode::Api => rmsg::MODE_API.get(),
    }
}

pub(super) fn new_combo() -> Combo {
    Combo::new("")
}

impl IntegrationsPage {
    /// The block's controls for a profile (none when the host has no lists).
    pub(super) fn radar_items(&self, e: &Integration) -> Vec<Item> {
        if !has_lists(e.kind) {
            return vec![];
        }
        let mut items = vec![Item::RadarReview, Item::RadarOpen, Item::RadarDrafts];
        if has_issues(e.kind) {
            items.push(Item::RadarIssues);
        }
        items.extend([Item::RadarNamespaces, Item::RadarMode]);
        items
    }

    /// The id of the connected profile the card shows, if any.
    fn radar_profile(&self, cfg: &Config) -> Option<String> {
        let i = self.selected_profile(cfg)?;
        let e = &cfg.integrations[i];
        (self.mode == Mode::Browse && e.user.is_some() && has_lists(e.kind)).then(|| e.id.clone())
    }

    /// Loads the Namespaces text when the card changed to another profile.
    pub(super) fn sync_radar(&mut self, cfg: &Config) {
        let Some(id) = self.radar_profile(cfg) else {
            return;
        };
        if self.radar_ns_for == id {
            return;
        }
        let names = cfg
            .integrations
            .iter()
            .find(|i| i.id == id)
            .map(|i| i.radar_options().namespaces)
            .unwrap_or_default();
        self.radar_ns = names.join(", ");
        self.radar_ns_for = id;
        self.radar_combo.close();
    }

    /// Writes the typed namespaces when they differ from the saved ones.
    pub(super) fn flush_radar(&mut self, env: &mut Env) {
        let Some(id) = self.radar_profile(env.config) else {
            return;
        };
        if self.radar_ns_for != id {
            return;
        }
        let names = parse_namespaces(&self.radar_ns);
        self.update_radar(env, |r| r.namespaces = names);
    }

    /// Applies `change` to the selected profile's options and saves, only
    /// when something changed.
    fn update_radar(&mut self, env: &mut Env, change: impl FnOnce(&mut IntegrationRadar)) {
        let Some(i) = self.selected_profile(env.config) else {
            return;
        };
        let before = env.config.integrations[i].radar_options();
        let mut after = before.clone();
        change(&mut after);
        if after == before {
            return;
        }
        env.config.integrations[i].radar = (after != IntegrationRadar::default()).then_some(after);
        Self::write_config(env);
    }

    pub(super) fn activate_radar(&mut self, item: Item, env: &mut Env) {
        match item {
            Item::RadarReview => self.update_radar(env, |r| r.review_requests ^= true),
            Item::RadarOpen => self.update_radar(env, |r| r.open_requests ^= true),
            Item::RadarDrafts => self.update_radar(env, |r| r.include_drafts ^= true),
            Item::RadarIssues => self.update_radar(env, |r| r.assigned_issues ^= true),
            Item::RadarNamespaces => self.step(1, env.config),
            Item::RadarMode => {
                self.fill_combo(env.config);
                self.radar_combo.open();
            }
            _ => {}
        }
    }

    fn fill_combo(&mut self, cfg: &Config) {
        let current = self
            .selected_profile(cfg)
            .map(|i| cfg.integrations[i].radar_options().review_mode)
            .unwrap_or_default();
        self.fill_combo_from(current);
    }

    /// The combobox's popup is open: it is modal.
    pub(crate) fn combo_open(&self) -> bool {
        self.radar_combo.is_open()
    }

    /// A key while the popup is open.
    pub(super) fn combo_key(&mut self, key: KeyEvent, env: &mut Env) {
        let outcome = self.radar_combo.on_key(key);
        self.combo_outcome(outcome, env);
    }

    /// A mouse event while the popup is open.
    pub(super) fn combo_mouse(&mut self, kind: MouseEventKind, col: u16, row: u16, env: &mut Env) {
        let event = crossterm::event::MouseEvent {
            kind,
            column: col,
            row,
            modifiers: KeyModifiers::NONE,
        };
        let outcome = self.radar_combo.on_mouse(event);
        self.combo_outcome(outcome, env);
    }

    fn combo_outcome(&mut self, outcome: ComboOutcome, env: &mut Env) {
        if let ComboOutcome::Picked(value) = outcome {
            if let Some(mode) = RemoteReviewMode::parse(&value) {
                self.update_radar(env, |r| r.review_mode = mode);
            }
        }
    }

    /// The keyboard lands on the first control of the connected profile's
    /// block (the palette's *Radar Options of a Profile*).
    pub(crate) fn focus_radar(&mut self, cfg: &Config) {
        let items = self.items(cfg);
        if let Some(at) = items.iter().position(|i| *i == Item::RadarReview) {
            self.item = at;
            self.zone = Zone::Body;
            self.scroll.want_radar = true;
        }
    }

    /// The first connected code host with account-wide lists.
    pub(crate) fn first_radar_profile(cfg: &Config) -> Option<&Integration> {
        cfg.integrations
            .iter()
            .find(|i| i.user.is_some() && has_lists(i.kind))
    }

    /// Paints the block from row `y` of the card.
    pub(super) fn render_radar(
        &mut self,
        frame: &mut Canvas,
        body: Rect,
        mut y: u16,
        e: &Integration,
        focused: bool,
        current: Option<Item>,
    ) {
        self.scroll.radar_top = Some(y);
        put(
            frame,
            body,
            body.x,
            y,
            vec![Span::styled(
                rmsg::HEADING.get(),
                Style::default()
                    .fg(th().ui.accent)
                    .add_modifier(Modifier::BOLD),
            )],
        );
        y += 1;
        if !has_lists(e.kind) {
            put(frame, body, body.x, y, muted(rmsg::NOT_AVAILABLE.get()));
            return;
        }
        let options = e.radar_options();
        for (item, on, text) in [
            (
                Item::RadarReview,
                options.review_requests,
                rmsg::REVIEW_REQUESTS.get(),
            ),
            (
                Item::RadarOpen,
                options.open_requests,
                rmsg::OPEN_REQUESTS.get(),
            ),
            (
                Item::RadarDrafts,
                options.include_drafts,
                rmsg::INCLUDE_DRAFTS.get(),
            ),
            (
                Item::RadarIssues,
                options.assigned_issues,
                rmsg::ASSIGNED_ISSUES.get(),
            ),
        ] {
            if item == Item::RadarIssues && !has_issues(e.kind) {
                continue;
            }
            let mark = if on { "[x]" } else { "[ ]" };
            self.control(
                frame,
                body,
                body.x,
                y,
                format!("{mark} {text}"),
                item,
                current,
                focused,
                false,
            );
            y += 1;
        }
        y += 1;
        put(frame, body, body.x, y, label(rmsg::NAMESPACES.get()));
        y += 1;
        let wide = body.width.saturating_sub(2).clamp(8, 80);
        self.text_field(
            frame,
            body,
            body.x,
            y,
            wide,
            Item::RadarNamespaces,
            current,
            focused,
            rmsg::NAMESPACES_HINT.get(),
        );
        y += 2;
        put(frame, body, body.x, y, label(rmsg::REVIEW_MODE.get()));
        y += 1;
        if !self.radar_combo.is_open() {
            self.fill_combo_from(options.review_mode);
        }
        if y < body.bottom() {
            let width = 28u16.min(body.width);
            let rect = Rect::new(body.x, y, width, 1);
            let is_focus = focused && self.zone == Zone::Body && current == Some(Item::RadarMode);
            self.radar_combo
                .render_field(frame.buffer_mut(), rect, is_focus, false);
            self.scroll.radar_field = Some(rect);
            self.hits.push((rect, Hit::Item(Item::RadarMode), true));
        }
    }

    fn fill_combo_from(&mut self, mode: RemoteReviewMode) {
        self.radar_combo.set_options(
            RemoteReviewMode::ALL
                .iter()
                .map(|m| ComboOption::new(mode_label(*m), m.as_str()))
                .collect(),
        );
        self.radar_combo.select(Some(mode.as_str().to_string()));
    }
}
