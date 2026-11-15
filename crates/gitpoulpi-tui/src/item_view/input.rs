//! Keys, the mouse, paste and the palette's commands of the item view.
//!
//! Every mouse action has its key (maintainer rule): the arrows and
//! `Ctrl+J` / `Ctrl+K` move between entries, `Tab` goes timeline, composer,
//! then the header's buttons, each button has a letter and a palette row.
//! Popups own the keyboard and the pointer innermost first: the discard
//! question, then the links menu.

use crossterm::event::{
    KeyCode, KeyEvent, KeyModifiers, MouseButton, MouseEvent, MouseEventKind,
};
use gitpoulpi_i18n::msg::item_view as m;

use super::{Bar, Focus, Hit, ItemEffect, ItemView};
use crate::combo::ComboOutcome;
use crate::conversation_ui::Act;
use crate::dialog_kit::{self, DismissAction, DismissKind};
use crate::item_cmd::ItemCmd;
use crate::md_editor::KeyUse;

impl ItemView {
    // -- the order Tab walks --------------------------------------------------------

    fn order(&self) -> Vec<Focus> {
        let mut order = vec![Focus::Timeline];
        if self.composer_available() && self.composer_open {
            order.push(Focus::Composer);
        }
        order.extend(Bar::ALL.iter().map(|b| Focus::Bar(*b)));
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
        if self.focus == Focus::Composer && !self.guard.is_open() && self.menu.is_none() {
            if let Some(md) = self.md.as_mut() {
                md.paste(text);
            }
        }
    }

    pub(crate) fn on_key(&mut self, key: KeyEvent) -> ItemEffect {
        self.message = None;
        if let Some(verdict) = self.guard.key(key) {
            return self.verdict(verdict);
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
            _ => {}
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
            KeyCode::Enter | KeyCode::Char(' ') if key.modifiers.is_empty() => {
                self.bar_press(bar)
            }
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
                None => {
                    let quote = self.conv.quote();
                    self.open_composer(quote.as_deref());
                    ItemEffect::None
                }
            },
            Act::NewComment => {
                if self.composer_available() {
                    self.open_composer(None);
                    ItemEffect::None
                } else {
                    ItemEffect::Message(m::WHY_NO_COMMENT.get().to_string())
                }
            }
            Act::Edit => ItemEffect::Message(
                self.avail()
                    .edit
                    .unwrap_or(m::WHY_WRITING_LATER.get())
                    .to_string(),
            ),
            Act::Delete => ItemEffect::Message(
                self.avail()
                    .delete
                    .unwrap_or(m::WHY_WRITING_LATER.get())
                    .to_string(),
            ),
            Act::Links => self.open_links_menu(),
            Act::CopyLink => match self.conv.selected_url() {
                Some(url) => ItemEffect::Message(crate::clipboard::copy_shared(
                    m::NOUN_LINK.get(),
                    &url,
                )),
                None => ItemEffect::Message(m::WHY_NO_URL.get().to_string()),
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

    /// Shows the composer and gives it the keyboard; `quote` goes in first.
    fn open_composer(&mut self, quote: Option<&str>) {
        self.ensure_composer();
        self.composer_open = true;
        self.focus = Focus::Composer;
        if let (Some(md), Some(quote)) = (self.md.as_mut(), quote) {
            md.append_block(quote);
        }
    }

    /// A send was asked for. Writing is the next slice: the reason says so.
    fn send(&mut self) -> ItemEffect {
        match self.send_reason() {
            Some(why) => ItemEffect::Message(why.to_string()),
            None => ItemEffect::None,
        }
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
            dialog_kit::Verdict::Discard => {
                if let Some(md) = self.md.as_mut() {
                    md.discard();
                }
                ItemEffect::Close
            }
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
        if let Some(v) = self.guard.mouse(mouse) {
            return self.verdict(v);
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
                    None => {}
                }
            }
            _ => {}
        }
        let in_md = self
            .md
            .as_ref()
            .is_some_and(|e| e.area().contains(pos.into()));
        if in_md || (self.focus == Focus::Composer && mouse.kind != MouseEventKind::Moved) {
            if mouse.kind == MouseEventKind::Down(MouseButton::Left) {
                self.focus = Focus::Composer;
            }
            if let Some(md) = self.md.as_mut() {
                md.on_mouse(mouse);
            }
            return ItemEffect::None;
        }
        if self.conv.area.contains(pos.into()) || self.conv.drag.held() {
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
        if self.composer_dirty() {
            self.guard.ask_with(
                gitpoulpi_i18n::msg::md_editor::DISCARD_TITLE.get(),
                gitpoulpi_i18n::msg::md_editor::DISCARD_BODY.get(),
            );
            return ItemEffect::None;
        }
        self.focus = Focus::Timeline;
        ItemEffect::None
    }

    // -- the palette ------------------------------------------------------------------------

    pub(crate) fn run(&mut self, cmd: ItemCmd) -> ItemEffect {
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
            ItemCmd::NewComment => self.act(Act::NewComment),
            ItemCmd::Links => self.act(Act::Links),
            ItemCmd::CopyLink => self.act(Act::CopyLink),
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
