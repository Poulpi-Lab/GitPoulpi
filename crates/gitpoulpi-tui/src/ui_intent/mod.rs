//! What outlives one window of the agent view-control feature (plan
//! the mcp-show-ui design). A tab switch ends the
//! window and starts another, so three things live here, in the process,
//! rather than in a window:
//!
//! - the **carried intent**: a `ui show tab` / `ui show home` that ends the
//!   window is handed to the next one, which applies it (it is then already
//!   on the target) and answers it on its own link;
//! - the **handled ids**, so the daemon re-sending an unanswered intent to a
//!   re-attached window does not apply it twice;
//! - the **snapshot stack** Back pops ([`snapshot`]), and the snapshot a Back
//!   across tabs asks the next window to restore.
//!
//! The applying itself is `event_loop/window/agent_view.rs`.

pub(crate) mod plan;
pub(crate) mod proposal;
mod snapshot;

use std::collections::VecDeque;
use std::sync::Mutex;
use std::time::{Duration, Instant};

use gitpoulpi_i18n::msg::ui_intent as msg;
use gitpoulpi_ipc::ui::{HostingItem, Proposal, UiIntent, UiOrigin};

pub(crate) use snapshot::{depth, hand_over_restore, pop, push, take_restore, ViewSnapshot};

/// A carried intent older than this is dropped (the window it was meant for
/// never came).
const CARRIED_FOR: Duration = Duration::from_secs(8);

static MARKED_TAB: Mutex<Option<std::path::PathBuf>> = Mutex::new(None);

/// The tab an agent's intent last changed the view of: drawn with a ✦ in the
/// tab bar until the user presses a key or clicks.
pub(crate) fn mark_tab(root: &std::path::Path) {
    *lock(&MARKED_TAB) = Some(root.to_path_buf());
    crate::redraw::poke();
}

pub(crate) fn marked_tab() -> Option<std::path::PathBuf> {
    lock(&MARKED_TAB).clone()
}

pub(crate) fn clear_marked_tab() {
    *lock(&MARKED_TAB) = None;
}

/// An intent a window hands to the next one.
pub(crate) struct Carried {
    /// The daemon's id when the agent is still waiting for the answer;
    /// `None` for one the user accepted from the chip.
    pub(crate) id: Option<u64>,
    pub(crate) intent: UiIntent,
    pub(crate) origin: UiOrigin,
    at: Instant,
}

static CARRIED: Mutex<Option<Carried>> = Mutex::new(None);
static HANDLED: Mutex<VecDeque<u64>> = Mutex::new(VecDeque::new());

fn lock<T>(m: &Mutex<T>) -> std::sync::MutexGuard<'_, T> {
    m.lock().unwrap_or_else(|e| e.into_inner())
}

/// Hands `intent` to the next window.
pub(crate) fn carry(id: Option<u64>, intent: UiIntent, origin: UiOrigin) {
    if let Some(id) = id {
        let mut handled = lock(&HANDLED);
        handled.push_back(id);
        while handled.len() > 32 {
            handled.pop_front();
        }
    }
    *lock(&CARRIED) = Some(Carried {
        id,
        intent,
        origin,
        at: Instant::now(),
    });
}

impl Carried {
    /// How long ago the previous window handed it over.
    pub(crate) fn age(&self) -> Duration {
        self.at.elapsed()
    }
}

/// Gives a carried intent back, unchanged and as old as it was, for a later
/// tick to take (the window is not ready for it yet).
pub(crate) fn hold_carried(carried: Carried) {
    *lock(&CARRIED) = Some(carried);
}

/// The intent the previous window handed over, if it is still fresh.
pub(crate) fn take_carried() -> Option<Carried> {
    lock(&CARRIED)
        .take()
        .filter(|c| c.at.elapsed() < CARRIED_FOR)
}

/// Whether `id` was carried over from a previous window (the daemon's
/// re-send of it is to be ignored).
pub(crate) fn is_handled(id: u64) -> bool {
    lock(&HANDLED).contains(&id)
}

/// What the chip and the toast call an intent.
pub(crate) fn describe(intent: &UiIntent) -> String {
    match intent {
        UiIntent::ShowTab { repo } => {
            let name = std::path::Path::new(&repo.0)
                .file_name()
                .map_or_else(|| repo.0.clone(), |n| n.to_string_lossy().into_owned());
            msg::tab(name)
        }
        UiIntent::ShowHome { filter: Some(f) } => msg::home_filter(f),
        UiIntent::ShowHome { filter: None } => msg::HOME.get().into(),
        UiIntent::ShowSettings { page, setting } => {
            let page = page.as_ref().map(|page| {
                serde_json::to_value(page)
                    .ok()
                    .and_then(|v| v.as_str().map(str::to_string))
                    .unwrap_or_default()
            });
            match (page, setting) {
                (Some(page), Some(setting)) => msg::settings_page_setting(page, setting),
                (Some(page), None) => msg::settings_page(page),
                (None, Some(setting)) => msg::settings_setting(setting),
                (None, None) => msg::SETTINGS.get().into(),
            }
        }
        UiIntent::ShowPalette { query } if query.is_empty() => msg::PALETTE.get().into(),
        UiIntent::ShowPalette { query } => msg::palette_query(query),
        UiIntent::ShowHelp { .. } => msg::HELP.get().into(),
        UiIntent::Back => msg::BACK.get().into(),
        UiIntent::ShowCommit { commit, .. } => {
            msg::commit(commit.to_string().chars().take(8).collect::<String>())
        }
        UiIntent::ShowRef { name, .. } => msg::ref_name(name),
        UiIntent::ShowSearch { query, .. } => msg::search(query),
        UiIntent::ShowHosting {
            item: HostingItem::Pr(n),
            ..
        } => msg::pull_request(n),
        UiIntent::ShowHosting {
            item: HostingItem::Issue(n),
            ..
        } => msg::issue(n),
        UiIntent::ShowWip { path: Some(p), .. } => msg::wip_path(p),
        UiIntent::ShowWip { .. } => msg::WIP.get().into(),
        UiIntent::ShowConflict { path, .. } => msg::conflicts(path),
        UiIntent::Propose { proposal, .. } => describe_proposal(proposal),
        other => other.kind().trim_start_matches("show_").replace('_', " "),
    }
}

/// What the chip and the banner call a proposal.
pub(crate) fn describe_proposal(proposal: &Proposal) -> String {
    let short =
        |oid: &gitpoulpi_core::domain::Oid| oid.to_string().chars().take(8).collect::<String>();
    match proposal {
        Proposal::ConflictResolution { path, .. } => msg::prop_resolution(path),
        Proposal::Reword { commit, .. } => msg::prop_reword(short(commit)),
        Proposal::SplitCommit { commit, .. } => msg::prop_split(short(commit)),
        Proposal::Rewrite { recipe } => msg::prop_rewrite(recipe.rules.len() as u64),
        Proposal::CommitMessage { .. } => msg::PROP_COMMIT_MESSAGE.get().into(),
        Proposal::Branch { name, start_point } => msg::prop_branch(name, short(start_point)),
        Proposal::Tag { name, target, .. } => msg::prop_tag(name, short(target)),
        Proposal::SequenceRun { id } => msg::prop_sequence(
            crate::sequence_run::registry::slot_of(id)
                .and_then(crate::sequence_run::registry::info)
                .map_or_else(|| id.clone(), |info| info.name),
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use gitpoulpi_core::domain::UiControl;
    use gitpoulpi_ipc::ui::SettingsPage;

    #[test]
    fn a_carried_intent_is_taken_once_and_its_id_is_remembered() {
        carry(Some(9_001), UiIntent::Back, UiOrigin::cli());
        assert!(is_handled(9_001));
        let c = take_carried().expect("carried");
        assert_eq!(c.id, Some(9_001));
        assert!(take_carried().is_none());
        assert!(!is_handled(9_002));
        let _ = UiControl::Auto;
    }

    #[test]
    fn describe_reads_in_the_ui_language() {
        let _fr = gitpoulpi_i18n::override_lang(gitpoulpi_i18n::Lang::Fr);
        assert_eq!(describe(&UiIntent::Back), "la vue précédente");
        assert_eq!(
            describe(&UiIntent::ShowSettings {
                page: Some(SettingsPage::UiCustomization),
                setting: Some("ui.minimap".into())
            }),
            "les Paramètres ui-customization / ui.minimap"
        );
        assert_eq!(
            describe(&UiIntent::ShowSearch {
                repo: gitpoulpi_core::domain::RepoId("/r".into()),
                query: "fix".into(),
            }),
            "une recherche de \u{ab}\u{a0}fix\u{a0}\u{bb}"
        );
    }

    #[test]
    fn describe_names_what_the_agent_wants_to_show() {
        assert_eq!(
            describe(&UiIntent::ShowSettings {
                page: Some(SettingsPage::UiCustomization),
                setting: Some("ui.minimap".into())
            }),
            "settings ui-customization / ui.minimap"
        );
        assert_eq!(
            describe(&UiIntent::ShowPalette {
                query: "pull".into()
            }),
            "the palette (pull)"
        );
        assert_eq!(
            describe(&UiIntent::ShowHosting {
                repo: gitpoulpi_core::domain::RepoId("/r".into()),
                item: HostingItem::Pr(7),
                view: Default::default(),
            }),
            "pull request #7"
        );
        assert_eq!(
            describe(&UiIntent::ShowWip {
                repo: gitpoulpi_core::domain::RepoId("/r".into()),
                path: None,
                staged: false,
                composer: true
            }),
            "the working changes"
        );
    }
}
