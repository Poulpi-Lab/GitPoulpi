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

/// Visible rows of the Namespaces text area.
const NS_ROWS: u16 = 4;

/// The entries typed in the Namespaces area: one per line (in plain mode a
/// comma also separates, the old single-line form), trimmed, empty ones and
/// repeats dropped.
pub(super) fn parse_namespaces(text: &str, regex: bool) -> Vec<String> {
    gitpoulpi_config::parse_namespace_text(text, regex)
}

/// The first invalid expression of the typed lines, when the area is in
/// regex mode: `"<line>: <why>"`.
fn regex_error(text: &str, regex: bool) -> Option<String> {
    if !regex {
        return None;
    }
    gitpoulpi_hosting::account::first_regex_error(&parse_namespaces(text, true))
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
        items.extend([Item::RadarNamespaces, Item::RadarRegex, Item::RadarMode]);
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
        self.radar_ns = names.join("\n");
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
        let regex = self
            .selected_profile(env.config)
            .is_some_and(|i| env.config.integrations[i].radar_options().namespaces_regex);
        // An invalid expression is shown under the area and not saved: the
        // last good list stays in force.
        if regex_error(&self.radar_ns, regex).is_some() {
            return;
        }
        let names = parse_namespaces(&self.radar_ns, regex);
        self.update_radar(env, |r| r.namespaces = names);
    }

    /// A key in the Namespaces area. Returns whether it was used; Up on the
    /// first line and Down on the last are not, so they leave the area.
    pub(super) fn namespaces_key(&mut self, key: &KeyEvent, env: &mut Env) -> bool {
        let item = Item::RadarNamespaces;
        let text = &mut self.radar_ns;
        if key.code == KeyCode::Enter && key.modifiers.is_empty() {
            self.editor.caret_mut(item).insert(text, "\n");
            return true;
        }
        let cursor = self.editor.caret_mut(item).cursor(text);
        let first = !text[..cursor].contains('\n');
        let last = !text[cursor..].contains('\n');
        let leaves = match key.code {
            KeyCode::Up => first && !key.modifiers.contains(KeyModifiers::SHIFT),
            KeyCode::Down => last && !key.modifiers.contains(KeyModifiers::SHIFT),
            _ => false,
        };
        if leaves || !self.editor.claims(item, text, key, true) {
            return false;
        }
        if let crate::text_edit::KeyEdit::Copy(s) | crate::text_edit::KeyEdit::Cut(s) =
            self.editor.on_key(item, text, key, true)
        {
            *env.message = Some(crate::clipboard::copy_shared(
                gitpoulpi_i18n::msg::input::WHAT_SELECTION.get(),
                &s,
            ));
        }
        self.error = None;
        true
    }

    /// Wheel over the Namespaces area scrolls its lines when they overflow.
    pub(super) fn namespaces_wheel(&mut self, kind: MouseEventKind, col: u16, row: u16) -> bool {
        let lines = self.radar_ns.split('\n').count();
        if lines <= NS_ROWS as usize
            || !matches!(kind, MouseEventKind::ScrollUp | MouseEventKind::ScrollDown)
        {
            return false;
        }
        let text = self.radar_ns.clone();
        let vrow = self.editor_row(row);
        self.editor
            .area_mouse(Item::RadarNamespaces, &text, kind, (col, vrow), false)
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
            Item::RadarRegex => self.toggle_regex(env),
            Item::RadarMode => {
                self.fill_combo(env.config);
                self.radar_combo.open();
            }
            _ => {}
        }
    }

    /// Flips "Regular expressions". Turning it on with a line that is not a
    /// valid expression is refused, with the reason in the status line.
    fn toggle_regex(&mut self, env: &mut Env) {
        let Some(i) = self.selected_profile(env.config) else {
            return;
        };
        let on = !env.config.integrations[i].radar_options().namespaces_regex;
        if let Some(why) = regex_error(&self.radar_ns, on) {
            *env.message = Some(rmsg::regex_invalid(&why));
            return;
        }
        let names = parse_namespaces(&self.radar_ns, on);
        self.update_radar(env, |r| {
            r.namespaces_regex = on;
            r.namespaces = names;
        });
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
        let help = if options.namespaces_regex {
            rmsg::NAMESPACES_REGEX_HELP.get()
        } else {
            rmsg::NAMESPACES_HELP.get()
        };
        y += put_wrapped(
            frame,
            body,
            body.x,
            y,
            help,
            Style::default().fg(th().ui.muted),
            3,
        );
        let wide = body.width.saturating_sub(2).clamp(8, 80);
        self.namespaces_area(frame, body, y, wide, current, focused);
        y += NS_ROWS;
        if let Some(why) = regex_error(&self.radar_ns, options.namespaces_regex) {
            put(
                frame,
                body,
                body.x,
                y,
                vec![Span::styled(
                    fit(&rmsg::regex_invalid(&why), body.width as usize),
                    Style::default().fg(th().status.danger),
                )],
            );
        }
        y += 1;
        let mark = if options.namespaces_regex {
            "[x]"
        } else {
            "[ ]"
        };
        self.control(
            frame,
            body,
            body.x,
            y,
            format!("{mark} {}", rmsg::REGEX.get()),
            Item::RadarRegex,
            current,
            focused,
            false,
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

    /// The Namespaces text area: `NS_ROWS` lines, one entry each, with a
    /// scrollbar on the right when the lines do not fit.
    fn namespaces_area(
        &mut self,
        frame: &mut Canvas,
        body: Rect,
        y: u16,
        width: u16,
        current: Option<Item>,
        focused: bool,
    ) {
        let item = Item::RadarNamespaces;
        let is_focus = focused && self.zone == Zone::Body && current == Some(item);
        let field_bg = crate::input_bg(is_focus, false);
        let width = width.min(body.right().saturating_sub(body.x));
        let rect = Rect::new(body.x, y, width, NS_ROWS);
        let text_w = width.saturating_sub(3).max(1);
        let mut top = 0;
        let lines = if self.radar_ns.is_empty() && !is_focus {
            // The empty area shows what to type.
            let hint = fit(rmsg::NAMESPACES_HINT.get(), text_w as usize);
            let mut rows = vec![Line::from(Span::styled(
                format!("{hint:<w$}", w = text_w as usize),
                Style::default().fg(th().ui.muted).bg(field_bg),
            ))];
            let blank = " ".repeat(text_w as usize);
            rows.resize(
                NS_ROWS as usize,
                Line::from(Span::styled(blank, Style::default().bg(field_bg))),
            );
            rows
        } else {
            // Not focused: the first lines show (the caret at the start).
            let caret = match self.editor.caret(item).cloned() {
                Some(caret) => caret,
                None => {
                    let mut caret = crate::text_edit::Caret::default();
                    caret.set_cursor(&self.radar_ns, 0, false);
                    caret
                }
            };
            let look = crate::text_field::EditLook::new(
                Style::default()
                    .fg(if is_focus {
                        th().status.warning
                    } else {
                        th().ui.text
                    })
                    .bg(field_bg),
                "\u{2588}",
            );
            let mut rows = crate::text_field::edit_lines(
                &self.radar_ns,
                Some(&caret),
                text_w as usize,
                NS_ROWS as usize,
                is_focus,
                look,
            );
            if let Some(real) = self.editor.caret(item) {
                real.scroll.set(caret.scroll.get());
                real.top.set(caret.top.get());
            }
            top = caret.top.get();
            let blank = " ".repeat(text_w as usize);
            rows.resize(
                NS_ROWS as usize,
                Line::from(Span::styled(blank, Style::default().bg(field_bg))),
            );
            rows
        };
        let total = self.radar_ns.split('\n').count();
        let buf = frame.buffer_mut();
        for (dy, line) in lines.into_iter().enumerate() {
            let row = y + dy as u16;
            if row >= body.bottom() {
                break;
            }
            let mut x = body.x;
            let pad = Style::default().bg(field_bg);
            if let Some(cell) = buf.cell_mut((x, row)) {
                cell.set_symbol(" ").set_style(pad);
            }
            x += 1;
            for span in line.spans {
                let (_, nx) =
                    buf.set_stringn(x, row, span.content.as_ref(), text_w as usize, span.style);
                x = nx;
            }
            // The scrollbar column: a thumb over a track, when lines overflow.
            let bar_x = rect.right().saturating_sub(1);
            let (glyph, style) = if total > NS_ROWS as usize {
                let rows = NS_ROWS as usize;
                let thumb = (rows * rows / total).max(1);
                let max_top = total - rows;
                let start = (top.min(max_top) * (rows - thumb))
                    .checked_div(max_top)
                    .unwrap_or(0);
                if dy >= start && dy < start + thumb {
                    (
                        "\u{2588}",
                        Style::default().fg(th().scrollbar.thumb).bg(field_bg),
                    )
                } else {
                    ("\u{2502}", Style::default().fg(th().ui.muted).bg(field_bg))
                }
            } else {
                (" ", pad)
            };
            if let Some(cell) = buf.cell_mut((bar_x, row)) {
                cell.set_symbol(glyph).set_style(style);
            }
            for fill in (x.max(body.x + 1 + text_w))..bar_x {
                if let Some(cell) = buf.cell_mut((fill, row)) {
                    cell.set_symbol(" ").set_style(pad);
                }
            }
        }
        self.editor.drawn(
            item,
            Rect::new(
                rect.x + 1,
                y,
                text_w,
                NS_ROWS.min(body.bottom().saturating_sub(y)),
            ),
            true,
        );
        self.hits.push((rect, Hit::Item(item), !is_focus));
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
