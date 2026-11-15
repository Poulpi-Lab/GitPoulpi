//! What a link of the conversation does.
//!
//! * an item of the repository's linked sources (`#7`, `!4`, `ABC-12`) opens
//!   in this view, and the one shown goes on the Back stack;
//! * a commit reveals in the Log when the repository is here, else it is a
//!   web address like the rest;
//! * every other address goes through `link_confirm`, which shows the real
//!   address before anything opens (the text is untrusted);
//! * a mention without a page and a label chip do nothing.

use gitpoulpi_config::ApiFamily;
use gitpoulpi_hosting::conversation::{ItemKey, ItemKind};
use gitpoulpi_hosting::refs::RefKind;
use gitpoulpi_hosting::xrefs::{ItemTarget as XItem, XrefTarget};
use gitpoulpi_i18n::msg::item_view as m;

use super::{ItemEffect, ItemTarget, ItemView, LinkMenu};
use crate::combo::{Combo, ComboOption};
use crate::md_render::LinkTarget;

/// `https://host/` and `https://HOST` are the same host.
fn same_base(a: &str, b: &str) -> bool {
    let norm = |s: &str| s.trim().trim_end_matches('/').to_ascii_lowercase();
    !a.trim().is_empty() && norm(a) == norm(b)
}

/// The words a link shows in the menu.
pub(super) fn link_label(link: &LinkTarget) -> String {
    match &link.xref {
        Some(XrefTarget::InApp(item)) => {
            let key = if item.key.is_empty() {
                item.number.to_string()
            } else {
                item.key.clone()
            };
            match item.kind {
                RefKind::Issue => m::link_issue(crate::md_render::sanitize(&key)),
                RefKind::PullRequest => m::link_request(crate::md_render::sanitize(&key)),
            }
        }
        Some(XrefTarget::Commit { sha, .. }) => {
            m::link_commit(crate::md_render::sanitize(&sha.chars().take(10).collect::<String>()))
        }
        Some(XrefTarget::Mention { login, .. }) => format!("@{}", crate::md_render::sanitize(login)),
        _ => crate::md_render::sanitize(&link.url),
    }
}

impl ItemView {
    /// The item `x` of a reference, when this view can read it: the profile,
    /// the project and the key to read.
    fn resolve(&self, x: &XItem) -> Option<ItemTarget> {
        let family = x.family;
        if matches!(family, ApiFamily::LocalMarkdown | ApiFamily::Trello) {
            return None;
        }
        let integration = if self.target.integration.kind.family() == family
            && (x.web_base.is_empty()
                || same_base(
                    &gitpoulpi_hosting::integration_web_base(&self.target.integration)
                        .unwrap_or_default(),
                    &x.web_base,
                ))
        {
            self.target.integration.clone()
        } else {
            self.target
                .all
                .iter()
                .find(|i| {
                    i.kind.family() == family
                        && same_base(
                            &gitpoulpi_hosting::integration_web_base(i).unwrap_or_default(),
                            &x.web_base,
                        )
                })?
                .clone()
        };
        let by_key = matches!(family, ApiFamily::Jira | ApiFamily::Linear | ApiFamily::YouTrack);
        let key_text = x.key.trim_start_matches(['#', '!']).to_string();
        if !by_key && x.number == 0 {
            return None;
        }
        let kind = match x.kind {
            RefKind::Issue => ItemKind::Issue,
            RefKind::PullRequest => ItemKind::PullRequest,
        };
        let key = ItemKey {
            kind,
            number: x.number,
            id: if by_key { key_text.clone() } else { String::new() },
        };
        let reference = if x.key.is_empty() {
            format!("#{}", x.number)
        } else {
            x.key.clone()
        };
        Some(ItemTarget {
            integration,
            all: self.target.all.clone(),
            project: if x.project.is_empty() {
                self.target.project.clone()
            } else {
                x.project.clone()
            },
            key,
            reference,
            title: String::new(),
            web_url: String::new(),
            repo_root: self.target.repo_root.clone(),
            bases: self.target.bases.clone(),
        })
    }

    /// A link was activated (a click, Enter, a menu row).
    pub(super) fn activate_link(&mut self, link: &LinkTarget) -> ItemEffect {
        match &link.xref {
            Some(XrefTarget::InApp(x)) => {
                if let Some(target) = self.resolve(x) {
                    self.push_item(target);
                    return ItemEffect::None;
                }
                self.ask(&link.url)
            }
            Some(XrefTarget::Commit { sha, url }) => {
                if self.target.repo_root.is_some() && sha.len() >= 7 {
                    ItemEffect::ShowCommit(sha.to_ascii_lowercase())
                } else {
                    match url.as_deref().filter(|u| !u.is_empty()) {
                        Some(u) => self.ask(u),
                        None => ItemEffect::None,
                    }
                }
            }
            Some(XrefTarget::Chip { .. }) => ItemEffect::None,
            Some(XrefTarget::Mention { url: None, .. }) => ItemEffect::None,
            Some(XrefTarget::Mention { url: Some(u), .. }) | Some(XrefTarget::Web(u)) => {
                self.ask(u)
            }
            None => self.ask(&link.url),
        }
    }

    fn ask(&mut self, url: &str) -> ItemEffect {
        if url.is_empty() {
            return ItemEffect::Message(m::NOTHING_TO_OPEN.get().to_string());
        }
        crate::link_confirm::ask(vec![url.to_string()]);
        ItemEffect::None
    }

    /// Opens the menu of the selected entry's links.
    pub(super) fn open_links_menu(&mut self) -> ItemEffect {
        let links = self.conv.selected_links();
        if links.is_empty() {
            return ItemEffect::Message(m::WHY_NO_LINKS.get().to_string());
        }
        let mut combo = Combo::new("").with_click_picks();
        combo.set_options(
            links
                .iter()
                .enumerate()
                .map(|(i, l)| ComboOption::new(link_label(l), i.to_string()))
                .collect(),
        );
        let area = self.conv.area;
        combo.set_anchor(ratatui::layout::Rect::new(
            area.x + 2,
            area.y + 1,
            area.width.saturating_sub(6).min(60),
            1,
        ));
        combo.open();
        self.menu = Some(LinkMenu { combo, links });
        ItemEffect::None
    }
}
