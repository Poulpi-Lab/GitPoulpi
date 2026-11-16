//! Reviewing a request from the Radar: which way, and the API review hosted
//! over the Radar.
//!
//! A request of the scope's own repositories opens its clone, as ever. A
//! request of an account-wide row goes the way the profile's `review_mode`
//! says (`ask` | `clone` | `api`), settled with whether a clone of its
//! repository is known (`way`): `ask` opens the [`ReviewChoice`] dialog.
//!
//! **The API review** is a [`ReviewView`] held by the Radar
//! (`RadarUi::api_review`) and drawn over it, in the Home window: a request
//! of a repository that is not cloned has no repository window to live in.
//! The Radar is already the first stop of the Home window's keys, mouse,
//! paste and poll, so the view is routed to first right here (`api_key`,
//! `api_mouse`, `api_poll`), and the palette reaches its commands through
//! [`RadarUi::review_avail`] / [`RadarUi::run_review`]. `Esc` and the view's
//! cross give the Radar back.

use gitpoulpi_config::RemoteReviewMode;
use ratatui::buffer::Buffer;

use super::review_choice::{ChoiceOutcome, Pick, ReviewChoice};
use super::vocab::ReviewRequest;
use super::*;
use crate::review_cmd::{ReviewAvail, ReviewCmd};
use crate::review_view::ReviewEffect;
#[cfg(feature = "review")]
use crate::review_view::ReviewView;

/// What a Review does for a request, once the profile's mode and the known
/// clone are in.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum Way {
    /// Ask with the choice dialog.
    Ask,
    /// Open the clone that is there.
    Local,
    /// Clone it, then review there.
    Clone,
    /// Review through the host's API.
    Api,
}

/// `mode` is the profile's `review_mode`; `cloned` says a clone is known.
pub(super) fn way(mode: RemoteReviewMode, cloned: bool) -> Way {
    match (mode, cloned) {
        (RemoteReviewMode::Ask, _) => Way::Ask,
        (RemoteReviewMode::Api, _) => Way::Api,
        (RemoteReviewMode::Clone, true) => Way::Local,
        (RemoteReviewMode::Clone, false) => Way::Clone,
    }
}

/// The open choice dialog and the row it is about.
pub(super) struct Asking {
    pub(super) dialog: ReviewChoice,
    pub(super) item: Item,
}

impl RadarUi {
    // -- the entry: Review, Review Through the API, Clone and Review ---------

    pub(super) fn review(&mut self, item: Item, act: Act) -> Outcome {
        if item.kind != ItemKind::Pr {
            return self.say(msg::radar::MSG_NOT_PR.get().to_string());
        }
        // No in-app review for the host (Azure DevOps, Bitbucket): the
        // request's page, and a line that says why.
        if !item.review {
            return self.review_in_browser(&item);
        }
        // A request of a repository of the scope: its own window reviews it.
        if item.integration.is_empty() {
            return match act {
                Act::Review => self.start_review(&item, Pick::Local),
                _ => self.say(msg::radar::MSG_REVIEW_HERE.get().to_string()),
            };
        }
        let cloned = item.repo.is_some();
        match act {
            Act::ReviewApi => self.start_review(&item, Pick::Api),
            Act::CloneReview if cloned => {
                self.message = None;
                self.start_review(&item, Pick::Local)
            }
            Act::CloneReview => self.start_review(&item, Pick::Clone),
            _ => match way(self.review_mode_of(&item.integration), cloned) {
                Way::Local => self.start_review(&item, Pick::Local),
                Way::Clone => self.start_review(&item, Pick::Clone),
                Way::Api => self.start_review(&item, Pick::Api),
                Way::Ask => {
                    self.close_popups();
                    let profile = self
                        .account_answers
                        .iter()
                        .find(|a| a.id == item.integration)
                        .map_or_else(|| item.integration.clone(), |a| a.label.clone());
                    let dialog = ReviewChoice::new(
                        Self::describe(&item),
                        item.project.clone(),
                        profile,
                        cloned,
                    );
                    self.asking = Some(Asking { dialog, item });
                    Outcome::None
                }
            },
        }
    }

    fn review_in_browser(&mut self, item: &Item) -> Outcome {
        if item.url.is_empty() {
            return self.say(msg::radar::MSG_NO_PAGE.get().to_string());
        }
        match crate::repo_menu::open_url(&item.url) {
            Ok(()) => self.say(msg::radar::msg_review_in_browser(&item.url)),
            Err(err) => self.say(err),
        }
    }

    /// The profile's `review_mode` (`ask` when the file cannot say).
    fn review_mode_of(&self, id: &str) -> RemoteReviewMode {
        self.config_dir
            .as_deref()
            .and_then(|dir| account_fs::integration_in(dir, id))
            .map(|i| i.radar_options().review_mode)
            .unwrap_or_default()
    }

    fn request_of(item: &Item) -> ReviewRequest {
        ReviewRequest {
            integration: item.integration.clone(),
            project: item.project.clone(),
            number: item.number,
            label: item.label.clone(),
            title: item.title.clone(),
            web_url: item.url.clone(),
            source_branch: item.branch.clone().unwrap_or_default(),
            target_branch: item.target.clone(),
            clone_url: item.clone_url.clone(),
        }
    }

    fn start_review(&mut self, item: &Item, pick: Pick) -> Outcome {
        match pick {
            Pick::Local => self.open_repo(
                item,
                Some(Pending::Review {
                    number: item.number,
                    repo: None,
                }),
            ),
            Pick::Api => Outcome::ApiReview(Box::new(Self::request_of(item))),
            Pick::Clone if item.clone_url.is_empty() => {
                self.say(msg::radar::msg_no_clone_url(&item.project))
            }
            Pick::Clone => Outcome::CloneReview(Box::new(Self::request_of(item))),
        }
    }

    // -- the choice dialog ---------------------------------------------------

    pub(super) fn choice_key(&mut self, key: KeyEvent) -> Outcome {
        let Some(asking) = self.asking.as_mut() else {
            return Outcome::None;
        };
        let outcome = asking.dialog.on_key(key);
        self.settle_choice(outcome)
    }

    pub(super) fn choice_mouse(&mut self, mouse: MouseEvent) -> Outcome {
        let Some(asking) = self.asking.as_mut() else {
            return Outcome::None;
        };
        let outcome = asking.dialog.on_mouse(mouse);
        self.settle_choice(outcome)
    }

    /// A badge of the dialog was pressed.
    pub(super) fn choice_press(&mut self, index: u8) -> Outcome {
        let Some(asking) = self.asking.as_mut() else {
            return Outcome::None;
        };
        let outcome = asking.dialog.nav_press(index);
        self.settle_choice(outcome)
    }

    fn settle_choice(&mut self, outcome: ChoiceOutcome) -> Outcome {
        match outcome {
            ChoiceOutcome::Stay => Outcome::None,
            ChoiceOutcome::Cancel => {
                self.asking = None;
                Outcome::None
            }
            ChoiceOutcome::Pick { pick, remember } => {
                let Some(asking) = self.asking.take() else {
                    return Outcome::None;
                };
                if remember && pick != Pick::Local {
                    if let Err(why) = self.remember(&asking.item.integration, pick) {
                        self.asking = Some(asking);
                        return self.say(msg::radar::msg_remember_failed(why));
                    }
                }
                self.start_review(&asking.item, pick)
            }
        }
    }

    /// Writes the profile's `review_mode` for a clone or API pick.
    fn remember(&mut self, id: &str, pick: Pick) -> Result<(), String> {
        let mode = match pick {
            Pick::Api => RemoteReviewMode::Api,
            _ => RemoteReviewMode::Clone,
        };
        let dir = self
            .config_dir
            .as_deref()
            .ok_or_else(|| msg::radar::WHY_NO_CONFIG.get().to_string())?;
        account_fs::set_review_mode(dir, id, mode)
    }

    pub(super) fn paint_choice(&mut self, buf: &mut Buffer, body: Rect) {
        if let Some(asking) = self.asking.as_mut() {
            asking.dialog.render(buf, body);
        }
    }

    // -- the API review hosted over the Radar --------------------------------

    /// Takes the view the event loop built for an [`Outcome::ApiReview`].
    #[cfg(feature = "review")]
    pub(crate) fn host_review(&mut self, view: Box<ReviewView>) {
        self.close_popups();
        self.asking = None;
        self.api_review = Some(view);
    }

    pub(crate) fn hosts_review(&self) -> bool {
        self.api_review.is_some()
    }

    /// The palette may open over the review (no box or picker is up in it).
    pub(crate) fn review_takes_palette(&self) -> bool {
        self.api_review
            .as_ref()
            .is_some_and(|v| v.takes_palette() || v.composer_avail().is_some())
    }

    /// What the palette may ask of the AI help of the review's open box.
    pub(crate) fn review_composer_avail(&self) -> Option<crate::composer_cmd::ComposerAvail> {
        self.api_review.as_ref().and_then(|v| v.composer_avail())
    }

    pub(crate) fn run_review_composer(
        &mut self,
        cmd: crate::composer_cmd::ComposerCmd,
    ) -> Option<String> {
        self.api_review.as_mut().and_then(|v| v.run_composer(cmd))
    }

    pub(crate) fn review_avail(&self) -> Option<ReviewAvail> {
        self.api_review.as_ref().map(|v| v.avail())
    }

    pub(crate) fn run_review(&mut self, cmd: ReviewCmd) -> Outcome {
        let Some(view) = self.api_review.as_mut() else {
            return Outcome::None;
        };
        let effect = view.run(cmd);
        self.on_review_effect(effect)
    }

    pub(super) fn api_key(&mut self, key: KeyEvent) -> Outcome {
        let Some(view) = self.api_review.as_mut() else {
            return Outcome::None;
        };
        let effect = view.on_key(key);
        self.on_review_effect(effect)
    }

    pub(super) fn api_mouse(&mut self, mouse: MouseEvent) -> Outcome {
        let Some(view) = self.api_review.as_mut() else {
            return Outcome::None;
        };
        let effect = view.on_mouse(mouse);
        self.on_review_effect(effect)
    }

    pub(crate) fn api_paste(&mut self, text: &str) {
        if let Some(view) = self.api_review.as_mut() {
            view.paste(text);
        }
    }

    /// Drains the view's workers; true when it has something new to draw.
    pub(super) fn api_poll(&mut self) -> bool {
        let Some(view) = self.api_review.as_mut() else {
            return false;
        };
        let effect = view.poll();
        if let Outcome::Message(text) = self.on_review_effect(effect) {
            self.said = Some(text);
        }
        true
    }

    pub(super) fn api_busy(&self) -> bool {
        self.api_review.as_ref().is_some_and(|v| v.busy())
    }

    pub(super) fn on_review_effect(&mut self, effect: ReviewEffect) -> Outcome {
        match effect {
            ReviewEffect::None => Outcome::None,
            ReviewEffect::Message(text) => self.say(text),
            ReviewEffect::Close => {
                self.api_review = None;
                Outcome::None
            }
            // A checkout needs the repository on disk.
            ReviewEffect::Checkout(_) => self.say(msg::radar::MSG_CLONE_FIRST.get().to_string()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_mode_and_the_known_clone_settle_the_way() {
        use RemoteReviewMode as M;
        assert_eq!(way(M::Ask, true), Way::Ask);
        assert_eq!(way(M::Ask, false), Way::Ask);
        assert_eq!(way(M::Clone, true), Way::Local);
        assert_eq!(way(M::Clone, false), Way::Clone);
        assert_eq!(way(M::Api, true), Way::Api);
        assert_eq!(way(M::Api, false), Way::Api);
    }
}
