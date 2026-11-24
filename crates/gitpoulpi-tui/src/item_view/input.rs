//! Keys, the mouse, paste and the palette's commands of the item view.
//!
//! Every mouse action has its key (maintainer rule): the arrows and
//! `Ctrl+J` / `Ctrl+K` move between entries, `Tab` goes timeline, composer,
//! then the header's buttons, each button has a letter and a palette row.
//! Popups own the keyboard and the pointer innermost first: the discard
//! question, then the links menu.

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers, MouseButton, MouseEvent, MouseEventKind};
use gitpoulpi_i18n::msg::item_view as m;

use super::{Bar, Compose, DeleteChoice, Discarding, Focus, Hit, ItemEffect, ItemView};
use crate::combo::ComboOutcome;
use crate::conversation_ui::write::{ReplyTo, WriteOp};
use crate::conversation_ui::Act;
use crate::dialog_kit::{self, ConfirmEvent, DismissAction, DismissKind};
use crate::item_cmd::ItemCmd;
use crate::md_editor::KeyUse;

impl ItemView {
    // -- the order Tab walks --------------------------------------------------------

    fn order(&self) -> Vec<Focus> {
        let mut order = vec![Focus::Timeline];
        if self.composer_available() && self.composer_open {
            order.push(Focus::Composer);
        }
        if !self.embedded {
            order.extend(Bar::ALL.iter().map(|b| Focus::Bar(*b)));
        }
        order
    }

    fn cycle(&mut self, back: bool) {
        let order = self.order();
        let at = order.iter().position(|f| *f == self.focus).unwrap_or(0);
        let n = order.len();
        let to = if back { (at + n - 1) % n } else { (at + 1) % n };
        self.focus = order[to];
    }

    // -- keys --------------------------------------------------------------------------

    pub(crate) fn paste(&mut self, text: &str) {
        if self.focus == Focus::Composer
            && !self.guard.is_open()
            && self.menu.is_none()
            && self.delete_ask.is_none()
            && !self.conv.is_writing()
        {
            if let Some(md) = self.md.as_mut() {
                md.paste(text);
            }
        }
    }

    /// A message stays in the view's own last row: the status line under the
    /// frame is not where the eyes are.
    fn note(&mut self, effect: ItemEffect) -> ItemEffect {
        // Inside another view the message goes to that view's status line.
        if self.embedded {
            return effect;
        }
        match effect {
            ItemEffect::Message(text) => {
                self.message = Some(text);
                ItemEffect::None
            }
            other => other,
        }
    }

    pub(crate) fn on_key(&mut self, key: KeyEvent) -> ItemEffect {
        self.message = None;
        let effect = self.key_inner(key);
        self.note(effect)
    }

    /// A key for the conversation shown inside another view. `None`: not
    /// the conversation's (the timeline has no use for it), the host view
    /// handles it. The composer, a popup or a question takes every key.
    pub(crate) fn embedded_key(&mut self, key: KeyEvent) -> Option<ItemEffect> {
        let owned = self.guard.is_open()
            || self.delete_ask.is_some()
            || self.menu.is_some()
            || self.focus == Focus::Composer;
        if !owned {
            let timeline_key = matches!(key.code, KeyCode::Tab | KeyCode::BackTab)
                && !key
                    .modifiers
                    .intersects(KeyModifiers::CONTROL | KeyModifiers::ALT)
                || self.conv.takes_key(&key);
            if !timeline_key {
                return None;
            }
        }
        Some(self.on_key(key))
    }

    /// No popup or question of the conversation is open over the page.
    pub(crate) fn takes_palette_for_mouse(&self) -> bool {
        !self.guard.is_open() && self.delete_ask.is_none() && self.menu.is_none()
    }

    /// The pointer is on something of the embedded conversation (or a popup
    /// of it is open and owns the pointer).
    pub(crate) fn owns_mouse(&self, mouse: &MouseEvent) -> bool {
        self.guard.is_open()
            || self.delete_ask.is_some()
            || self.menu.is_some()
            || self.conv.drag.held()
            || self.conv.text.has_selection()
            || self.frame.contains((mouse.column, mouse.row).into())
    }

    /// The pointer left the embedded conversation: nothing in it is hovered.
    pub(crate) fn clear_hover(&mut self) {
        self.hover = None;
        self.conv.hover = None;
    }

    fn key_inner(&mut self, key: KeyEvent) -> ItemEffect {
        if let Some(verdict) = self.guard.key(key) {
            return self.verdict(verdict);
        }
        if let Some((confirm, _)) = self.delete_ask.as_mut() {
            let event = confirm.on_key(key);
            return self.delete_event(event);
        }
        if let Some(menu) = self.menu.as_mut() {
            let outcome = menu.combo.on_key(key);
            return self.menu_outcome(outcome);
        }
        match self.focus {
            Focus::Timeline => self.timeline_key(key),
            Focus::Composer => self.composer_key(key),
            Focus::Bar(bar) => self.bar_key(bar, key),
        }
    }

    fn timeline_key(&mut self, key: KeyEvent) -> ItemEffect {
        match key.code {
            KeyCode::Tab if key.modifiers.is_empty() => {
                self.cycle(false);
                return ItemEffect::None;
            }
            KeyCode::BackTab => {
                self.cycle(true);
                return ItemEffect::None;
            }
            _ => {}
        }
        match self.conv.on_key(key) {
            Some(act) => self.act(act),
            None => ItemEffect::None,
        }
    }

    fn composer_key(&mut self, key: KeyEvent) -> ItemEffect {
        match key.code {
            KeyCode::Tab if key.modifiers.is_empty() => {
                self.cycle(false);
                return ItemEffect::None;
            }
            KeyCode::BackTab => {
                self.cycle(true);
                return ItemEffect::None;
            }
            KeyCode::Esc => {
                self.focus = Focus::Timeline;
                return ItemEffect::None;
            }
            // The twin of the internal note checkbox.
            KeyCode::Char('i') if key.modifiers == KeyModifiers::ALT => {
                return self.toggle_internal();
            }
            _ => {}
        }
        if self.conv.is_writing() {
            return ItemEffect::Message(m::WHY_SENDING.get().to_string());
        }
        let Some(md) = self.md.as_mut() else {
            self.focus = Focus::Timeline;
            return ItemEffect::None;
        };
        match md.on_key(key) {
            KeyUse::Submit => self.send(),
            KeyUse::Handled | KeyUse::Pass => ItemEffect::None,
        }
    }

    fn bar_key(&mut self, bar: Bar, key: KeyEvent) -> ItemEffect {
        let at = Bar::ALL.iter().position(|b| *b == bar).unwrap_or(0);
        match key.code {
            KeyCode::Left if key.modifiers.is_empty() => {
                self.focus = Focus::Bar(Bar::ALL[(at + Bar::ALL.len() - 1) % Bar::ALL.len()]);
                ItemEffect::None
            }
            KeyCode::Right if key.modifiers.is_empty() => {
                self.focus = Focus::Bar(Bar::ALL[(at + 1) % Bar::ALL.len()]);
                ItemEffect::None
            }
            KeyCode::Tab if key.modifiers.is_empty() => {
                self.cycle(false);
                ItemEffect::None
            }
            KeyCode::BackTab => {
                self.cycle(true);
                ItemEffect::None
            }
            KeyCode::Enter | KeyCode::Char(' ') if key.modifiers.is_empty() => self.bar_press(bar),
            KeyCode::Esc => {
                self.focus = Focus::Timeline;
                ItemEffect::None
            }
            KeyCode::F(5) => self.act(Act::Refresh),
            _ => ItemEffect::None,
        }
    }

    // -- what a button or a key does ------------------------------------------------

    fn bar_press(&mut self, bar: Bar) -> ItemEffect {
        match bar {
            Bar::Back => self.act(Act::Back),
            Bar::Refresh => self.act(Act::Refresh),
            Bar::Browser => self.act(Act::OpenBrowser),
            Bar::Events => self.act(Act::ToggleEvents),
        }
    }

    pub(super) fn act(&mut self, act: Act) -> ItemEffect {
        match act {
            Act::Handled => ItemEffect::None,
            Act::Reply => match self.conv.reply_reason() {
                Some(why) => ItemEffect::Message(why.to_string()),
                None => match self.conv.reply_to() {
                    ReplyTo::Discussion { id, label } => self.begin(
                        Compose::Reply {
                            discussion: id,
                            label,
                        },
                        String::new(),
                        None,
                    ),
                    ReplyTo::Quote => self.quote_reply(),
                },
            },
            Act::Quote => match self.avail().quote {
                Some(why) => ItemEffect::Message(why.to_string()),
                None => self.quote_reply(),
            },
            Act::NewComment => {
                if self.composer_available() {
                    self.begin(Compose::New, String::new(), None)
                } else {
                    ItemEffect::Message(m::WHY_NO_COMMENT.get().to_string())
                }
            }
            Act::Edit => match self.avail().edit {
                Some(why) => ItemEffect::Message(why.to_string()),
                None => {
                    let Some(c) = self.conv.writable_comment() else {
                        return ItemEffect::None;
                    };
                    let text = crate::conversation_ui::rows::markdown_of(&c.body);
                    let mode = Compose::Edit {
                        id: c.id.clone(),
                        original: text.clone(),
                    };
                    self.begin(mode, text, None)
                }
            },
            Act::Delete => match self.avail().delete {
                Some(why) => ItemEffect::Message(why.to_string()),
                None => {
                    let Some(id) = self.conv.writable_comment().map(|c| c.id.clone()) else {
                        return ItemEffect::None;
                    };
                    self.ask_delete(id);
                    ItemEffect::None
                }
            },
            Act::Links => self.open_links_menu(),
            Act::OpenDiagram => self.open_diagram_menu(),
            Act::OpenCode => match self.avail().code {
                Some(why) => ItemEffect::Message(why.to_string()),
                None => self
                    .conv
                    .selected_code()
                    .map_or(ItemEffect::None, ItemEffect::OpenCode),
            },
            Act::CopyLink => match self.conv.selected_url() {
                Some(url) => {
                    ItemEffect::Message(crate::clipboard::copy_shared(m::NOUN_LINK.get(), &url))
                }
                None => ItemEffect::Message(m::WHY_NO_URL.get().to_string()),
            },
            Act::CopyText => match self.conv.copy_text() {
                Some(text) => {
                    ItemEffect::Message(crate::clipboard::copy_shared(m::NOUN_TEXT.get(), &text))
                }
                None => ItemEffect::Message(m::WHY_NO_TEXT.get().to_string()),
            },
            Act::OpenBrowser => {
                let url = self
                    .conv
                    .selected_url()
                    .or_else(|| Some(self.target.web_url.clone()))
                    .filter(|u| !u.is_empty());
                match url {
                    Some(u) => ItemEffect::OpenUrl(u),
                    None => ItemEffect::Message(m::WHY_NO_URL.get().to_string()),
                }
            }
            Act::Refresh => {
                self.conv.refresh();
                ItemEffect::None
            }
            Act::LoadOlder => {
                if self.conv.older.is_some() {
                    self.conv.load_older();
                    ItemEffect::None
                } else {
                    ItemEffect::Message(m::WHY_NO_OLDER.get().to_string())
                }
            }
            Act::ToggleEvents => {
                if self.conv.caps.as_ref().is_some_and(|c| c.events) {
                    let on = !self.conv.events;
                    self.conv.set_events(on);
                    super::save_events(&self.target, on);
                    ItemEffect::None
                } else {
                    ItemEffect::Message(m::WHY_NO_EVENTS.get().to_string())
                }
            }
            Act::Back => {
                if self.pop_item() {
                    ItemEffect::None
                } else {
                    ItemEffect::Message(m::WHY_NO_BACK.get().to_string())
                }
            }
            Act::Close => self.request_close(DismissKind::Esc),
            Act::Link(link) => self.activate_link(&link),
        }
    }

    /// Shows the composer in `mode` and gives it the keyboard; `quote` goes
    /// in after what is there.
    fn begin(&mut self, mode: Compose, text: String, quote: Option<&str>) -> ItemEffect {
        match self.start_compose(mode, text) {
            Err(why) => ItemEffect::Message(why.to_string()),
            Ok(()) => {
                if let (Some(md), Some(quote)) = (self.md.as_mut(), quote) {
                    md.append_block(quote);
                }
                ItemEffect::None
            }
        }
    }

    /// A new comment with the selected entry quoted in it.
    fn quote_reply(&mut self) -> ItemEffect {
        let quote = self.conv.quote();
        self.begin(Compose::New, String::new(), quote.as_deref())
    }

    /// The internal note checkbox (hosts that have such notes).
    pub(super) fn toggle_internal(&mut self) -> ItemEffect {
        if let Some(why) = self.avail().internal {
            return ItemEffect::Message(why.to_string());
        }
        self.internal = !self.internal;
        ItemEffect::None
    }

    fn ask_delete(&mut self, id: String) {
        let confirm = dialog_kit::Confirm::new(
            m::DELETE_TITLE.get(),
            m::DELETE_BODY.get(),
            vec![
                (m::DELETE_KEEP.get().to_string(), DeleteChoice::Keep),
                (m::DELETE_CONFIRM.get().to_string(), DeleteChoice::Delete),
            ],
            DeleteChoice::Keep,
        )
        .primary(DeleteChoice::Delete);
        self.delete_ask = Some((confirm, id));
    }

    fn delete_event(&mut self, event: ConfirmEvent<DeleteChoice>) -> ItemEffect {
        match event {
            ConfirmEvent::Stay => {}
            ConfirmEvent::Chose(choice) => {
                if let Some((_, id)) = self.delete_ask.take() {
                    if choice == DeleteChoice::Delete {
                        self.conv.submit(WriteOp::Delete { id });
                    }
                }
            }
        }
        ItemEffect::None
    }

    /// Sends what the comment box holds: a new comment, an answer or the
    /// new text of an own comment. The entry shows at once; a refusal puts
    /// it back with the text still in the box.
    fn send(&mut self) -> ItemEffect {
        if let Some(why) = self.send_reason() {
            return ItemEffect::Message(why.to_string());
        }
        let Some(md) = self.md.as_ref() else {
            return ItemEffect::None;
        };
        let body = md.text().trim_end().to_string();
        let op = match &self.compose {
            Compose::New => WriteOp::Post {
                body,
                internal: self.internal
                    && self.conv.caps.as_ref().is_some_and(|c| c.internal_notes),
            },
            Compose::Reply { discussion, .. } => WriteOp::Reply {
                discussion_id: discussion.clone(),
                body,
            },
            Compose::Edit { id, .. } => WriteOp::Edit {
                id: id.clone(),
                body,
            },
        };
        self.compose_error = None;
        self.conv.submit(op);
        ItemEffect::None
    }

    fn menu_outcome(&mut self, outcome: ComboOutcome) -> ItemEffect {
        let Some(menu) = self.menu.as_ref() else {
            return ItemEffect::None;
        };
        let open = menu.combo.is_open();
        let picked = match outcome {
            ComboOutcome::Picked(value) => value.parse::<usize>().ok(),
            _ => None,
        };
        let link = picked.and_then(|i| menu.links.get(i).cloned());
        if !open {
            self.menu = None;
        }
        match link {
            Some(l) => self.activate_link(&l),
            None => ItemEffect::None,
        }
    }

    // -- closing ------------------------------------------------------------------------

    fn request_close(&mut self, kind: DismissKind) -> ItemEffect {
        if self.embedded {
            // The host view closes itself; the text in the box stays as a
            // draft for the next visit.
            return ItemEffect::Close;
        }
        self.discarding = Discarding::Close;
        match dialog_kit::dismiss(&*self, kind) {
            DismissAction::Close => {
                if let Some(md) = self.md.as_mut() {
                    md.discard();
                }
                ItemEffect::Close
            }
            DismissAction::Swallow => {
                self.guard.close.request_flash();
                ItemEffect::None
            }
            DismissAction::Confirm => {
                self.guard.ask_with(
                    gitpoulpi_i18n::msg::md_editor::DISCARD_TITLE.get(),
                    gitpoulpi_i18n::msg::md_editor::DISCARD_BODY.get(),
                );
                ItemEffect::None
            }
        }
    }

    fn verdict(&mut self, v: dialog_kit::Verdict) -> ItemEffect {
        match v {
            dialog_kit::Verdict::Discard => match self.discarding {
                Discarding::Close => {
                    if let Some(md) = self.md.as_mut() {
                        md.discard();
                    }
                    ItemEffect::Close
                }
                Discarding::Composer => {
                    self.discarding = Discarding::Close;
                    self.reset_composer();
                    self.focus = Focus::Timeline;
                    ItemEffect::None
                }
            },
            dialog_kit::Verdict::Stay => ItemEffect::None,
        }
    }

    // -- the mouse ------------------------------------------------------------------------

    fn hit_at(&self, mouse: &MouseEvent) -> Option<Hit> {
        self.hits
            .iter()
            .find(|(r, _)| r.contains((mouse.column, mouse.row).into()))
            .map(|(_, h)| *h)
    }

    pub(crate) fn on_mouse(&mut self, mouse: MouseEvent) -> ItemEffect {
        let effect = self.mouse_inner(mouse);
        self.note(effect)
    }

    fn mouse_inner(&mut self, mouse: MouseEvent) -> ItemEffect {
        if let Some(v) = self.guard.mouse(mouse) {
            return self.verdict(v);
        }
        if let Some((confirm, _)) = self.delete_ask.as_mut() {
            let event = confirm.on_mouse(mouse);
            return self.delete_event(event);
        }
        if let Some(menu) = self.menu.as_mut() {
            let outcome = menu.combo.on_mouse(mouse);
            return self.menu_outcome(outcome);
        }
        let pos = (mouse.column, mouse.row);
        let hit = self.hit_at(&mouse);
        match mouse.kind {
            MouseEventKind::Moved => {
                self.hover = hit;
                self.guard.close.set_hover(pos.0, pos.1);
            }
            MouseEventKind::Down(MouseButton::Left) => {
                self.message = None;
                if self.guard.close.hit(pos.0, pos.1) {
                    return self.request_close(DismissKind::Cross);
                }
                match hit {
                    Some(Hit::Bar(bar)) => {
                        self.focus = Focus::Bar(bar);
                        return self.bar_press(bar);
                    }
                    Some(Hit::Send) => {
                        self.focus = Focus::Composer;
                        return self.send();
                    }
                    Some(Hit::Cancel) => return self.cancel_composer(),
                    Some(Hit::Internal) => {
                        self.focus = Focus::Composer;
                        return self.toggle_internal();
                    }
                    None => {}
                }
            }
            _ => {}
        }
        let in_md = self
            .md
            .as_ref()
            .is_some_and(|e| e.area().contains(pos.into()));
        // A drag or a release that left the field still belongs to it; a press
        // elsewhere is a click on the timeline.
        let keeps = self.focus == Focus::Composer
            && matches!(mouse.kind, MouseEventKind::Drag(_) | MouseEventKind::Up(_));
        if in_md || keeps {
            if mouse.kind == MouseEventKind::Down(MouseButton::Left) {
                self.focus = Focus::Composer;
            }
            if let Some(md) = self.md.as_mut() {
                md.on_mouse(mouse);
            }
            return ItemEffect::None;
        }
        if self.conv.area.contains(pos.into())
            || self.conv.drag.held()
            || self.conv.text.has_selection()
        {
            if mouse.kind == MouseEventKind::Down(MouseButton::Left) {
                self.focus = Focus::Timeline;
            }
            if let Some(act) = self.conv.on_mouse(mouse) {
                return self.act(act);
            }
        }
        ItemEffect::None
    }

    /// Cancel under the composer: an untouched composer is just put away, a
    /// typed one asks first.
    fn cancel_composer(&mut self) -> ItemEffect {
        if self.conv.is_writing() {
            return ItemEffect::Message(m::WHY_SENDING.get().to_string());
        }
        if self.composer_dirty() {
            self.discarding = Discarding::Composer;
            self.guard.ask_with(
                gitpoulpi_i18n::msg::md_editor::DISCARD_TITLE.get(),
                gitpoulpi_i18n::msg::md_editor::DISCARD_BODY.get(),
            );
            return ItemEffect::None;
        }
        if self.compose != Compose::New {
            self.reset_composer();
        }
        self.compose_error = None;
        self.focus = Focus::Timeline;
        ItemEffect::None
    }

    // -- the palette ------------------------------------------------------------------------

    pub(crate) fn run(&mut self, cmd: ItemCmd) -> ItemEffect {
        self.message = None;
        let effect = self.run_inner(cmd);
        self.note(effect)
    }

    fn run_inner(&mut self, cmd: ItemCmd) -> ItemEffect {
        if let Some(why) = cmd.unavailable(Some(self.avail())) {
            return ItemEffect::Message(why.to_string());
        }
        match cmd {
            ItemCmd::Back => self.act(Act::Back),
            ItemCmd::Refresh => self.act(Act::Refresh),
            ItemCmd::LoadOlder => self.act(Act::LoadOlder),
            ItemCmd::PrevEntry => {
                self.conv.step(-1);
                ItemEffect::None
            }
            ItemCmd::NextEntry => {
                self.conv.step(1);
                ItemEffect::None
            }
            ItemCmd::NextUnread => {
                self.conv.next_unread();
                ItemEffect::None
            }
            ItemCmd::Reply => self.act(Act::Reply),
            ItemCmd::Quote => self.act(Act::Quote),
            ItemCmd::ToggleInternal => self.toggle_internal(),
            ItemCmd::NewComment => self.act(Act::NewComment),
            ItemCmd::Links => self.act(Act::Links),
            ItemCmd::OpenDiagram => self.act(Act::OpenDiagram),
            ItemCmd::OpenCode => self.act(Act::OpenCode),
            ItemCmd::CopyLink => self.act(Act::CopyLink),
            ItemCmd::CopyText => self.act(Act::CopyText),
            ItemCmd::OpenBrowser => self.act(Act::OpenBrowser),
            ItemCmd::Edit => self.act(Act::Edit),
            ItemCmd::Delete => self.act(Act::Delete),
            ItemCmd::ToggleEvents => self.act(Act::ToggleEvents),
            ItemCmd::TogglePreview => {
                self.ensure_composer();
                if let Some(md) = self.md.as_mut() {
                    md.toggle_preview();
                }
                ItemEffect::None
            }
            ItemCmd::ToggleSplit => {
                self.ensure_composer();
                if let Some(md) = self.md.as_mut() {
                    md.toggle_split();
                }
                ItemEffect::None
            }
            ItemCmd::Send => self.send(),
            ItemCmd::Close => self.request_close(DismissKind::Esc),
        }
    }
}

impl dialog_kit::Dismiss for ItemView {
    fn is_dirty(&self) -> bool {
        self.composer_dirty()
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

/// Whether `key` is a bare chord (no Ctrl / Alt).
#[allow(dead_code)]
fn bare(key: &KeyEvent) -> bool {
    !key.modifiers
        .intersects(KeyModifiers::CONTROL | KeyModifiers::ALT)
}
