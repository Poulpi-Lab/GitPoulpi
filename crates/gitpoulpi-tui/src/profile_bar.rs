//! The toolbar's per-repository profile combobox: which `[[profiles]]` entry
//! signs this repository's commits.
//!
//! Shown only with two or more profiles (or when a workspace's default profile
//! applies). The first row, "Default", inherits: the active workspace's
//! profile when the repository is one of its members, else
//! `general.active_profile`; the others pick one by id. The choice is stored
//! in the repository's own state (`RepoPreferences::profile`), so it follows
//! the repository and never the machine's global default.
//!
//! **Who wins**: the by-hand choice of this repository, then the active
//! workspace's profile (the repository must be in it), then the global
//! default. The field says which one it is: "Work - from workspace Acme".
//!
//! Like `open_in_bar` this is process-wide state (the window restarts on every
//! tab switch) and the popup is modal while open: it owns the keyboard and the
//! mouse, and its rows lead the frame's hit list. The keyboard twin is the
//! palette's `Repository: Change Profile...` ([`open`]).

use std::path::Path;
use std::sync::Mutex;

use crossterm::event::{KeyEvent, MouseEvent};
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::Frame;

use crate::chrome::HitTarget;
use crate::combo::{Combo, ComboOption, ComboOutcome};
use crate::display_width;
use gitpoulpi_config::{Config, Profile};
use gitpoulpi_i18n::msg::profile_bar as msg;

/// The widest the field gets, in cells.
const MAX_FIELD: u16 = 40;

#[derive(Default)]
struct State {
    profiles: Vec<Profile>,
    /// `general.active_profile`.
    global: String,
    /// This repository's own choice (`None` inherits).
    repo: Option<String>,
    /// The active workspace, when this repository is one of its members:
    /// its name and its default profile id.
    workspace: Option<(String, Option<String>)>,
    /// What `load_repo` was given, so a workspace change can re-resolve.
    session: String,
    repo_root: Option<std::path::PathBuf>,
    popup: Option<Combo>,
    anchor: Option<Rect>,
}

static STATE: Mutex<State> = Mutex::new(State {
    profiles: Vec::new(),
    global: String::new(),
    repo: None,
    workspace: None,
    session: String::new(),
    repo_root: None,
    popup: None,
    anchor: None,
});

/// Where the profile in effect comes from.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum Source {
    /// This repository's own choice.
    Repository,
    /// The active workspace's default (its name).
    Workspace(String),
    /// `general.active_profile`.
    Global,
    /// No profile: the identity of the system's own configuration.
    None,
}

fn with<R>(f: impl FnOnce(&mut State) -> R) -> R {
    let mut guard = STATE.lock().unwrap_or_else(|e| e.into_inner());
    f(&mut guard)
}

impl State {
    fn find(&self, id: &str) -> Option<&Profile> {
        self.profiles.iter().find(|p| p.id == id)
    }

    /// The profile the *inherited* choice gives: the active workspace's
    /// default when it names an existing profile, else the global one.
    fn inherited(&self) -> (Option<&Profile>, Source) {
        if let Some((name, Some(id))) = &self.workspace {
            if let Some(p) = self.find(id) {
                return (Some(p), Source::Workspace(name.clone()));
            }
        }
        match self.find(&self.global) {
            Some(p) => (Some(p), Source::Global),
            None => (None, Source::None),
        }
    }

    /// The profile commits use here and where it comes from: the
    /// repository's choice if it still names a profile, else what is
    /// inherited.
    fn resolved(&self) -> (Option<&Profile>, Source) {
        match self.repo.as_deref().and_then(|id| self.find(id)) {
            Some(p) => (Some(p), Source::Repository),
            None => self.inherited(),
        }
    }

    fn effective(&self) -> Option<&Profile> {
        self.resolved().0
    }

    fn default_label(&self) -> String {
        match self.inherited() {
            (Some(p), Source::Workspace(ws)) => {
                gitpoulpi_i18n::msg::workspaces::default_from_workspace(&p.name, ws)
            }
            (Some(p), _) => msg::default_named(&p.name),
            (None, _) => msg::DEFAULT_GIT.get().to_string(),
        }
    }

    /// The field's text: the repository's profile, else the inherited one
    /// with its source.
    fn shown_label(&self) -> String {
        match self.resolved() {
            (Some(p), Source::Repository) => p.name.clone(),
            (Some(p), Source::Workspace(ws)) => {
                gitpoulpi_i18n::msg::workspaces::from_workspace(&p.name, ws)
            }
            _ => self.default_label(),
        }
    }

    /// The workspace default applies here (so the field is worth showing
    /// even with a single profile).
    fn workspace_applies(&self) -> bool {
        matches!(self.inherited().1, Source::Workspace(_))
    }

    fn options(&self) -> Vec<ComboOption> {
        let mut out = vec![ComboOption::new(self.default_label(), "")];
        out.extend(
            self.profiles
                .iter()
                .map(|p| ComboOption::new(p.name.clone(), p.id.clone())),
        );
        out
    }
}

/// Publishes the profiles and the global default (startup and every live
/// settings change). Republishes the commit author.
pub(crate) fn install(config: &Config) {
    with(|s| {
        s.profiles = config.profiles.clone();
        s.global = config.general.active_profile.clone();
    });
    crate::live_settings::publish_author(effective());
}

/// Reads this repository's stored choice and the active workspace's default
/// (the window's startup).
pub(crate) fn load_repo(repo_root: &Path, session: &str) {
    let choice = gitpoulpi_session::RepoState::load(&crate::remotes_ui::repo_state_path(repo_root))
        .ok()
        .and_then(|state| state.preferences.profile);
    with(|s| {
        s.repo = choice;
        s.session = session.to_string();
        s.repo_root = Some(repo_root.to_path_buf());
    });
    refresh_workspace();
}

/// Re-reads which workspace is active and whether this repository is in it
/// (startup, and whenever a workspace changed), then republishes the commit
/// author.
pub(crate) fn refresh_workspace() {
    let (session, root) = with(|s| (s.session.clone(), s.repo_root.clone()));
    let workspace = match root {
        Some(root) if !session.is_empty() => crate::workspace_ui::store::active(&session)
            .filter(|w| w.contains(&root))
            .map(|w| (w.name, w.profile)),
        _ => None,
    };
    with(|s| s.workspace = workspace);
    crate::live_settings::publish_author(effective());
}

/// Where the profile in effect comes from.
#[cfg(test)]
pub(crate) fn source() -> Source {
    with(|s| s.resolved().1)
}

/// The profile this repository's commits use, if any.
pub(crate) fn effective() -> Option<Profile> {
    with(|s| s.effective().cloned())
}

/// Two profiles or more, or a workspace's default is in effect: the combobox
/// is worth showing.
pub(crate) fn visible() -> bool {
    with(|s| s.profiles.len() >= 2 || s.workspace_applies())
}

/// Cells the field takes: the longest label, the field's padding and arrow.
pub(crate) fn field_width() -> u16 {
    with(|s| {
        let widest = s
            .options()
            .iter()
            .map(|o| display_width(&o.label))
            .max()
            .unwrap_or(0) as u16;
        (widest + 4).min(MAX_FIELD)
    })
}

/// Paints the field into `rect` and remembers it as the popup's anchor.
pub(crate) fn render_field(buf: &mut Buffer, rect: Rect, hovered: bool) {
    let (label, open) = with(|s| {
        (
            s.shown_label(),
            s.popup.as_ref().is_some_and(Combo::is_open),
        )
    });
    let mut field = Combo::new(&label);
    field.set_options(vec![ComboOption::plain(label.clone())]);
    field.select(Some(label));
    field.render_field(buf, rect, open, hovered);
    with(|s| s.anchor = Some(rect));
}

pub(crate) fn is_open() -> bool {
    with(|s| s.popup.as_ref().is_some_and(Combo::is_open))
}

/// Opens the popup on the repository's current choice.
pub(crate) fn open() {
    with(|s| {
        let mut combo = Combo::new("");
        combo.set_options(s.options());
        combo.select(Some(
            s.repo
                .clone()
                .filter(|id| s.find(id).is_some())
                .unwrap_or_default(),
        ));
        combo.open();
        s.popup = Some(combo);
    });
}

fn settle(slot: &mut Option<Combo>, outcome: ComboOutcome) -> Option<String> {
    let pick = match outcome {
        ComboOutcome::Picked(value) | ComboOutcome::Custom(value) => Some(value),
        _ => None,
    };
    if slot.as_ref().is_some_and(|c| !c.is_open()) {
        *slot = None;
    }
    pick
}

/// A key while the popup is open (it swallows them all). `Some(id)` is a
/// pick: `""` is the default row.
pub(crate) fn on_key(key: KeyEvent) -> Option<String> {
    with(|s| {
        let outcome = s.popup.as_mut()?.on_key(key);
        settle(&mut s.popup, outcome)
    })
}

/// A mouse event while the popup is open (it swallows them all).
pub(crate) fn on_mouse(mouse: MouseEvent) -> Option<String> {
    with(|s| {
        let outcome = s.popup.as_mut()?.on_mouse(mouse);
        settle(&mut s.popup, outcome)
    })
}

pub(crate) fn paste(text: &str) {
    with(|s| {
        if let Some(combo) = s.popup.as_mut() {
            combo.paste(text);
        }
    });
}

/// Paints the open popup and puts its rows, then the whole popup, at the
/// front of `hits`.
pub(crate) fn render_popup(
    frame: &mut Frame,
    hovered_row: Option<usize>,
    hits: &mut Vec<(Rect, HitTarget)>,
) {
    let bounds = frame.area();
    with(|s| {
        let (Some(anchor), Some(combo)) = (s.anchor, s.popup.as_mut()) else {
            return;
        };
        combo.set_anchor(anchor);
        combo.set_hover(hovered_row);
        combo.render_popup(frame.buffer_mut(), bounds);
        let mut front: Vec<(Rect, HitTarget)> = combo
            .item_rects()
            .into_iter()
            .map(|(rect, row)| (rect, HitTarget::ProfileItem(row)))
            .collect();
        let popup = combo.popup_rect();
        if popup.width > 0 {
            front.push((popup, HitTarget::ProfilePopup));
        }
        hits.splice(0..0, front);
    });
}

/// What choosing a row did.
pub(crate) struct Chosen {
    /// The status line's answer.
    pub(crate) message: String,
    /// The identity to offer writing into the repository's git config, when
    /// the chosen profile has a complete one.
    pub(crate) identity: Option<(String, String)>,
}

/// Stores a pick (`""` = default) in the repository's state and republishes
/// the commit author. `Err` is the status line's complaint.
pub(crate) fn choose(repo_root: &Path, id: &str) -> Result<Chosen, String> {
    let path = crate::remotes_ui::repo_state_path(repo_root);
    let mut state = gitpoulpi_session::RepoState::load(&path).map_err(|e| e.to_string())?;
    let choice = (!id.is_empty()).then(|| id.to_string());
    state.preferences.profile = choice.clone();
    state.save(&path).map_err(|e| e.to_string())?;
    with(|s| s.repo = choice.clone());
    crate::live_settings::publish_author(effective());
    let profile = effective();
    let identity = profile
        .as_ref()
        .filter(|p| !p.author_name.is_empty() && !p.author_email.is_empty())
        .map(|p| (p.author_name.clone(), p.author_email.clone()));
    let message = match (&choice, &profile) {
        (None, _) => msg::CLEARED.get().to_string(),
        (Some(_), Some(p)) => msg::set_only(&p.name),
        (Some(_), None) => msg::CLEARED.get().to_string(),
    };
    Ok(Chosen { message, identity })
}

/// The tooltip of the field.
pub(crate) fn tooltip() -> &'static str {
    msg::TOOLTIP.get()
}

/// The name a status line gives the profile now in effect.
pub(crate) fn effective_name() -> Option<String> {
    effective().map(|p| p.name)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The state is process-wide: tests that set it take turns.
    static TEST_LOCK: Mutex<()> = Mutex::new(());

    fn config(active: &str) -> Config {
        let mut c = Config {
            profiles: vec![Profile::new("home", "Home"), Profile::new("work", "Work")],
            ..Config::default()
        };
        c.general.active_profile = active.into();
        c
    }

    /// The resolution the field shows is the one `Config::profile_for` gives
    /// (the CLI's), with the repository's own choice on top.
    #[test]
    fn who_wins_is_the_repository_then_the_workspace_then_the_global_default() {
        let _guard = TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        use gitpoulpi_session::Workspace;
        let cfg = config("home");
        install(&cfg);
        let mut ws = Workspace {
            id: "acme".into(),
            name: "Acme".into(),
            profile: Some("work".into()),
            ..Workspace::default()
        };
        ws.repos.push("/r/in".into());
        let set = |ws: &Workspace, repo: Option<&str>, member: &str| {
            with(|s| {
                s.repo = repo.map(str::to_string);
                s.workspace = ws
                    .contains(std::path::Path::new(member))
                    .then(|| (ws.name.clone(), ws.profile.clone()));
            });
        };
        // Neither a choice nor membership: the global default.
        set(&ws, None, "/r/out");
        assert_eq!(
            (effective().unwrap().id, source()),
            ("home".into(), Source::Global)
        );
        // A member: the workspace's profile, named in the field.
        set(&ws, None, "/r/in");
        assert_eq!(
            (effective().unwrap().id, source()),
            ("work".into(), Source::Workspace("Acme".into()))
        );
        assert_eq!(
            ws.profile_for(&cfg, std::path::Path::new("/r/in"))
                .unwrap()
                .id,
            "work",
            "the CLI's resolution agrees"
        );
        assert_eq!(with(|s| s.shown_label()), "Work - from workspace Acme");
        assert_eq!(
            with(|s| s.default_label()),
            "Default: Work (workspace Acme)"
        );
        // The by-hand choice of this repository wins over both.
        set(&ws, Some("home"), "/r/in");
        assert_eq!(
            (effective().unwrap().id, source()),
            ("home".into(), Source::Repository)
        );
        assert_eq!(with(|s| s.shown_label()), "Home");
        // A workspace profile naming nothing stops applying, quietly.
        ws.profile = Some("deleted".into());
        set(&ws, None, "/r/in");
        assert_eq!(
            (effective().unwrap().id, source()),
            ("home".into(), Source::Global)
        );
        with(|s| s.workspace = None);
        install(&Config::default());
    }

    #[test]
    fn the_repository_choice_beats_the_global_default_and_a_stale_id_falls_back() {
        let _guard = TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        install(&config("home"));
        with(|s| s.repo = Some("work".into()));
        assert_eq!(effective().unwrap().id, "work");
        with(|s| s.repo = Some("gone".into()));
        assert_eq!(effective().unwrap().id, "home");
        with(|s| s.repo = None);
        assert!(visible());
        install(&Config::default());
        assert!(!visible());
        assert!(effective().is_none());
    }
}
