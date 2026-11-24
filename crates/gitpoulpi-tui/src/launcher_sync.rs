//! Keeps the shortcuts we made up to date: their icon in the logo color and
//! their content (the terminal they start, the exe path).
//!
//! The event loop calls [`poll`] every iteration with nothing but a lock to
//! take: it compares the colorway the title bar wears (`about::way_index`)
//! and the `ui.launch_terminal` choice (both published at start and on every
//! live config reload, so a `gitpoulpi config set` reaches it while a TUI
//! runs) with what the shortcuts were last synced for. A change is debounced
//! (the About picker can change the color on each click) and then applied on
//! a thread (`launcher::sync_shortcuts`), because [`Env::real`] lists `PATH`
//! and the sync writes files. Only shortcuts that carry our marker and read
//! `Outdated` are rewritten; nothing is ever created.
//!
//! The first poll counts as a change: shortcuts left stale while no TUI ran
//! (the CLI, a hand-edited config, a moved exe) are fixed at start.
//! The CLI itself does not sync (`gitpoulpi-commands` links no TUI code);
//! `gitpoulpi desktop shortcut status` and Settings > Launcher also do.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Mutex;
use std::time::{Duration, Instant};

use gitpoulpi_config::installation::{self, Effective, Identity};

/// How long the color must hold still before the icons are rewritten while
/// the About window is open (closing it applies at once).
pub(crate) const DEBOUNCE: Duration = Duration::from_secs(1);

#[derive(Default)]
struct Watch {
    /// What the shortcuts were last synced for (color and terminal).
    synced: Option<String>,
    /// A value seen but not applied yet, and since when.
    wanted: Option<(String, Instant)>,
}

impl Watch {
    /// The value to apply now, if any.
    fn step(&mut self, slug: &str, now: Instant, settling: bool) -> Option<String> {
        if self.synced.as_deref() == Some(slug) {
            self.wanted = None;
            return None;
        }
        if self.wanted.as_ref().map(|(s, _)| s.as_str()) != Some(slug) {
            self.wanted = Some((slug.to_string(), now));
        }
        let since = self.wanted.as_ref().map_or(now, |(_, t)| *t);
        if self.synced.is_none() || !settling || now.duration_since(since) >= DEBOUNCE {
            self.wanted = None;
            self.synced = Some(slug.to_string());
            return Some(slug.to_string());
        }
        None
    }

    fn deadline(&self) -> Option<Instant> {
        self.wanted.as_ref().map(|(_, t)| *t + DEBOUNCE)
    }
}

/// `ui.launch_terminal` as last published.
static TERMINAL: Mutex<String> = Mutex::new(String::new());

/// `ui.launcher_build = "newest"` with dev mode on, as last published.
static NEWEST: AtomicBool = AtomicBool::new(false);

/// What this installation uses (its entry, else `ui.*` when primary, else the
/// kind's default), as last published. Never another installation's values.
static EFFECTIVE: Mutex<Option<Effective>> = Mutex::new(None);

/// The version the launcher name carries when the setting asks for it.
const VERSION: &str = env!("CARGO_PKG_VERSION");

/// This installation's effective values in `config`. A config file lacking
/// this installation's entry (a pinned id keeps its seeded entry in memory
/// only) falls back to the entry `seeded` holds. Pure.
pub(crate) fn effective_in(
    config: &gitpoulpi_config::Config,
    identity: &Identity,
    seeded: &gitpoulpi_config::Config,
) -> Effective {
    if config.installation.settings.contains_key(&identity.id) {
        return installation::effective(config, identity);
    }
    let mut merged = config.clone();
    if let Some(entry) = seeded.installation.settings.get(&identity.id) {
        merged
            .installation
            .settings
            .insert(identity.id.clone(), entry.clone());
    }
    installation::effective(&merged, identity)
}

/// [`effective_in`] for the running installation. The first call identifies
/// the exe and may seed its entry on disk (`installation::current`): the
/// startup publish does it before the loop, never the render thread.
pub(crate) fn effective_of(config: &gitpoulpi_config::Config) -> Effective {
    let (identity, seeded) = running();
    effective_in(config, &identity, &seeded)
}

/// The running installation and its seeded config. Tests never identify the
/// machine's own installation (that would read and seed the real config).
pub(crate) fn running() -> (Identity, gitpoulpi_config::Config) {
    #[cfg(test)]
    {
        let cur = crate::preferences::fake_current(false);
        (cur.identity, cur.config)
    }
    #[cfg(not(test))]
    {
        let cur = installation::current();
        (cur.identity.clone(), cur.config.clone())
    }
}

/// `config` as this installation shows it: its own colour keys (theme and
/// graph colours) in `ui.*`, never what another installation stored there.
pub(crate) fn own_config(config: &gitpoulpi_config::Config) -> gitpoulpi_config::Config {
    installation::overlay_look(config, &effective_of(config))
}

/// [`own_config`] of the file at `path`.
pub(crate) fn load_own(path: &std::path::Path) -> Option<gitpoulpi_config::Config> {
    gitpoulpi_config::Config::load(path)
        .ok()
        .map(|c| own_config(&c))
}

/// Writes one `ui.*` key the way this installation owns it: a colour key
/// (theme, graph colours) goes to its own entry unless it is the primary,
/// everything else is the shared key. Returns the stored value as JSON.
pub(crate) fn write_ui_key(
    path: &std::path::Path,
    key: &str,
    value: &str,
    identity: &Identity,
) -> gitpoulpi_config::Result<serde_json::Value> {
    match key
        .strip_prefix("ui.")
        .filter(|_| installation::is_look_key(key))
    {
        Some(short) => installation::set_look_in_file(path, identity, short, value)
            .map(serde_json::Value::String),
        None => gitpoulpi_config::set_in_file(path, key, value),
    }
}

/// The colour this installation wears in `config`.
pub(crate) fn effective_color(config: &gitpoulpi_config::Config) -> String {
    effective_of(config).logo_color
}

/// The effective values last published (startup and every live reload).
pub(crate) fn effective() -> Effective {
    EFFECTIVE
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .clone()
        .unwrap_or_else(|| installation::current().effective.clone())
}

/// What the shortcuts were last synced for: the colour, the launcher name
/// (with the version when the setting asks), the terminal and the build.
pub(crate) fn sync_key(slug: &str, name: &str, terminal: &str, newest: bool) -> String {
    format!("{slug}\n{terminal}\n{newest}\n{name}")
}

/// Publishes the terminal choice, the launcher build and this installation's
/// values (startup and every live settings change).
pub(crate) fn install(config: &gitpoulpi_config::Config) {
    *EFFECTIVE.lock().unwrap_or_else(|e| e.into_inner()) = Some(effective_of(config));
    *TERMINAL.lock().unwrap_or_else(|e| e.into_inner()) = config.ui.launch_terminal.clone();
    let exe = std::env::current_exe().unwrap_or_default();
    NEWEST.store(
        gitpoulpi_config::dev_mode::launcher_newest(config, &exe),
        Ordering::SeqCst,
    );
}

static STATE: Mutex<Option<Watch>> = Mutex::new(None);
static BUSY: AtomicBool = AtomicBool::new(false);
static UPDATED: AtomicBool = AtomicBool::new(false);
/// The one-time note about legacy launchers is due.
static LEGACY_NOTE: AtomicBool = AtomicBool::new(false);

fn with<R>(f: impl FnOnce(&mut Watch) -> R) -> R {
    let mut guard = STATE.lock().unwrap_or_else(|e| e.into_inner());
    f(guard.get_or_insert_with(Watch::default))
}

/// When the loop must look in again to apply a debounced change.
pub(crate) fn next_deadline() -> Option<Instant> {
    with(|s| s.deadline())
}

/// One loop iteration. Returns the footer note once a sync rewrote something.
pub(crate) fn poll() -> Option<String> {
    if !BUSY.load(Ordering::SeqCst) {
        let slug = crate::logo_colors::ways()[crate::about::way_index()]
            .slug
            .clone();
        let terminal = TERMINAL.lock().unwrap_or_else(|e| e.into_inner()).clone();
        let newest = NEWEST.load(Ordering::SeqCst);
        let e = effective();
        let name = installation::launcher_name(&e.name, e.name_with_version, VERSION);
        let key = sync_key(&slug, &name, &terminal, newest);
        let apply = with(|s| s.step(&key, Instant::now(), crate::about::is_open()));
        if apply.is_some() {
            BUSY.store(true, Ordering::SeqCst);
            std::thread::spawn(move || {
                let mut env = crate::launcher::Env::real();
                env.newest = newest;
                if crate::launcher::sync_shortcuts(&env, &terminal, &slug) {
                    UPDATED.store(true, Ordering::SeqCst);
                }
                if crate::launcher::legacy_note_due(&env, &terminal) {
                    LEGACY_NOTE.store(true, Ordering::SeqCst);
                }
                // The running window wears the colour too (Windows).
                crate::window_ctl::refresh_window_icon();
                BUSY.store(false, Ordering::SeqCst);
                crate::redraw::poke();
            });
        }
    }
    if LEGACY_NOTE.swap(false, Ordering::SeqCst) {
        return Some(gitpoulpi_i18n::msg::launcher::LEGACY_NOTE.get().to_string());
    }
    UPDATED.swap(false, Ordering::SeqCst).then(|| {
        gitpoulpi_i18n::msg::launcher::ICONS_UPDATED
            .get()
            .to_string()
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_first_look_applies_and_a_repeat_does_nothing() {
        let (mut s, t) = (Watch::default(), Instant::now());
        assert_eq!(s.step("a", t, true).as_deref(), Some("a"));
        assert_eq!(s.step("a", t, true), None);
    }

    #[test]
    fn a_color_that_keeps_changing_waits_until_it_holds_or_the_picker_closes() {
        let (mut s, t) = (Watch::default(), Instant::now());
        s.step("a", t, true);
        let at = |ms| t + Duration::from_millis(ms);
        assert_eq!(s.step("b", at(100), true), None);
        assert_eq!(s.step("c", at(900), true), None, "restarts the wait");
        assert_eq!(s.step("c", at(1800), true), None);
        assert_eq!(s.deadline(), Some(at(900) + DEBOUNCE));
        assert_eq!(s.step("c", at(1950), true).as_deref(), Some("c"));
        // Closing the window applies at once.
        assert_eq!(s.step("d", at(2000), true), None);
        assert_eq!(s.step("d", at(2010), false).as_deref(), Some("d"));
        assert_eq!(s.deadline(), None);
    }

    #[test]
    fn flipping_back_to_the_synced_color_cancels_the_wait() {
        let (mut s, t) = (Watch::default(), Instant::now());
        s.step("a", t, true);
        s.step("b", t, true);
        assert_eq!(s.step("a", t, false), None);
        assert_eq!(s.deadline(), None);
    }

    #[test]
    fn sync_key_changes_with_the_version_when_asked() {
        let key = |with: bool, version: &str| {
            let name = installation::launcher_name("GitPoulpi (dev)", with, version);
            sync_key("sun", &name, "kitty", false)
        };
        assert_ne!(key(true, "1.2.0"), key(true, "1.3.0"));
        assert_eq!(key(false, "1.2.0"), key(false, "1.3.0"));
        assert_eq!(
            key(true, "1.2.0"),
            key(true, "1.2.9"),
            "the patch is not shown"
        );
    }

    #[test]
    fn another_installations_colour_change_does_not_recolour_this_one() {
        use gitpoulpi_config::InstallationEntry;
        let mine = Identity {
            id: "aaaaaaaaaaaaaaaa".into(),
            kind: Default::default(),
            root: "/a".into(),
            channel: None,
        };
        let other = "bbbbbbbbbbbbbbbb".to_string();
        let entry = |color: &str| InstallationEntry {
            logo_color: color.into(),
            ..Default::default()
        };
        let mut config = gitpoulpi_config::Config::default();
        config
            .installation
            .settings
            .insert(mine.id.clone(), entry("teal"));
        config
            .installation
            .settings
            .insert(other.clone(), entry("sun"));
        let seeded = config.clone();
        let before = effective_in(&config, &mine, &seeded).logo_color;
        // The other installation changes its colour; a shared `ui.*` moves too.
        config.installation.settings.insert(other, entry("green"));
        config.ui.logo_color = "orange".into();
        assert_eq!(effective_in(&config, &mine, &seeded).logo_color, before);
        assert_eq!(before, "teal");
    }

    #[test]
    fn a_new_terminal_with_the_same_color_is_a_change_applied_at_once() {
        let (mut s, t) = (Watch::default(), Instant::now());
        assert!(s.step("red\nkitty", t, false).is_some());
        assert_eq!(s.step("red\nkitty", t, false), None);
        assert_eq!(s.step("red\nfoot", t, false).as_deref(), Some("red\nfoot"));
    }
}
