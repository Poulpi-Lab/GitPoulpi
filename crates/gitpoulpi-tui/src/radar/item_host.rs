//! The conversation view over the Radar, for an issue or a request whose
//! repository is not cloned.
//!
//! The view needs the integration profile and the project, not a checkout
//! (`ItemTarget::repo_root` is `None`), so the Radar hosts it the way it
//! hosts the API review: the event loop builds the view from an
//! [`Outcome::ApiConversation`] (it knows the configured profiles) and hands
//! it back to [`RadarUi::host_item`]; the view then owns the keyboard and the
//! mouse until it closes. What needs a repository (revealing a commit in the
//! Log) is not offered: a commit link opens on the host, and the palette's
//! reasons say so.

use super::*;
use crate::item_cmd::{ItemAvail, ItemCmd};
use crate::item_view::{ItemEffect, ItemView};

impl RadarUi {
    /// The Conversation action for a row of a repository that is not
    /// cloned: the request the event loop turns into a view.
    pub(super) fn conversation_without_clone(&mut self, item: &Item) -> Outcome {
        if item.integration.is_empty() || item.project.is_empty() {
            return self.say(msg::radar::MSG_CLONE_FIRST.get().to_string());
        }
        Outcome::ApiConversation(Box::new(ConversationRequest {
            integration: item.integration.clone(),
            project: item.project.clone(),
            number: item.number,
            pr: item.kind == ItemKind::Pr,
            label: item.label.clone(),
            title: item.title.clone(),
            web_url: item.url.clone(),
        }))
    }

    /// Takes the view the event loop built for an
    /// [`Outcome::ApiConversation`].
    pub(crate) fn host_item(&mut self, view: Box<ItemView>) {
        self.close_popups();
        self.asking = None;
        self.api_item = Some(view);
    }

    pub(crate) fn hosts_item(&self) -> bool {
        self.api_item.is_some()
    }

    /// The palette may open over the conversation (no field or popup of it
    /// has the keyboard).
    pub(crate) fn item_takes_palette(&self) -> bool {
        self.api_item
            .as_ref()
            .is_some_and(|v| v.takes_palette() || v.composer_avail().is_some())
    }

    /// What the palette may ask of the composer's AI help.
    pub(crate) fn item_composer_avail(&self) -> Option<crate::composer_cmd::ComposerAvail> {
        self.api_item.as_ref().and_then(|v| v.composer_avail())
    }

    pub(crate) fn run_item_composer(
        &mut self,
        cmd: crate::composer_cmd::ComposerCmd,
    ) -> Option<String> {
        self.api_item.as_mut().and_then(|v| v.run_composer(cmd))
    }

    pub(crate) fn item_avail(&self) -> Option<ItemAvail> {
        self.api_item
            .as_ref()
            .map(|v| v.avail())
            // The hosted review's Overview is the same conversation.
            .or_else(|| self.api_review.as_ref().and_then(|v| v.item_avail()))
    }

    /// Another client wrote to an item shown over the Radar.
    pub(crate) fn on_hosting_changed(&mut self, change: &gitpoulpi_ipc::HostingChangeArgs) {
        if let Some(view) = self.api_item.as_mut() {
            view.on_hosting_changed(change);
        }
        if let Some(view) = self.api_review.as_mut() {
            view.on_hosting_changed(change);
        }
    }

    pub(crate) fn run_item(&mut self, cmd: ItemCmd) -> Outcome {
        let Some(view) = self.api_item.as_mut() else {
            let Some(review) = self.api_review.as_mut() else {
                return Outcome::None;
            };
            let effect = review.run_item(cmd);
            return self.on_review_effect(effect);
        };
        let effect = view.run(cmd);
        self.on_item_effect(effect)
    }

    pub(super) fn item_key(&mut self, key: KeyEvent) -> Outcome {
        let Some(view) = self.api_item.as_mut() else {
            return Outcome::None;
        };
        let effect = view.on_key(key);
        self.on_item_effect(effect)
    }

    pub(super) fn item_mouse(&mut self, mouse: MouseEvent) -> Outcome {
        let Some(view) = self.api_item.as_mut() else {
            return Outcome::None;
        };
        let effect = view.on_mouse(mouse);
        self.on_item_effect(effect)
    }

    pub(crate) fn item_paste(&mut self, text: &str) {
        if let Some(view) = self.api_item.as_mut() {
            view.paste(text);
        }
    }

    /// Drains the view's worker; true when it has something new to draw.
    pub(super) fn item_poll(&mut self) -> bool {
        let Some(view) = self.api_item.as_mut() else {
            return false;
        };
        let effect = view.poll();
        if let Outcome::Message(text) = self.on_item_effect(effect) {
            self.said = Some(text);
        }
        true
    }

    pub(super) fn item_busy(&self) -> bool {
        self.api_item.as_ref().is_some_and(|v| v.busy())
    }

    /// When the hosted view next wants a frame (its 60 s refresh).
    pub(super) fn item_deadline(&self) -> Option<Instant> {
        self.api_item.as_ref().and_then(|v| v.next_deadline())
    }

    pub(super) fn paint_hosted_item(&mut self, buf: &mut ratatui::buffer::Buffer, body: Rect) {
        if let Some(view) = self.api_item.as_mut() {
            view.render(body, buf);
        }
    }

    fn on_item_effect(&mut self, effect: ItemEffect) -> Outcome {
        match effect {
            ItemEffect::None => Outcome::None,
            ItemEffect::Message(text) => self.say(text),
            ItemEffect::Close => {
                self.api_item = None;
                Outcome::None
            }
            ItemEffect::OpenUrl(url) => match crate::repo_menu::open_url(&url) {
                Ok(()) => self.say(msg::hosting_ui::opened_in_browser(&url)),
                Err(err) => self.say(err),
            },
            ItemEffect::OpenDiagram(source) => {
                crate::mermaid_popup::open_script(&source);
                Outcome::None
            }
            ItemEffect::OpenCode(_) => self.say(msg::item_view::WHY_NO_CODE_HERE.get().to_string()),
            // No repository here: the view never offers it (a commit link
            // opens on the host), and nothing else can ask.
            ItemEffect::ShowCommit(_) => self.say(msg::radar::MSG_CLONE_FIRST.get().to_string()),
        }
    }
}
