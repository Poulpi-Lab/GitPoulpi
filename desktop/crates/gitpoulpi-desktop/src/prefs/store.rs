//! The settings as an application global: read anywhere, written through the shared setters, re-read
//! when the files change on disk.

use std::time::Duration;

use gitpoulpi_app::dates::DateSettings;
use gitpoulpi_config::installation::{self, Identity};
use gitpoulpi_config::{Config, ConfigPaths, DesktopColumnWidths, DesktopConfig, DesktopUiConfig};
use gitpoulpi_i18n::Lang;
use gitpoulpi_session::RecentRepos;
use gpui_kit::{App, Global};

use super::load::{anonymous, dates_of, Loaded};
use super::watch::{Fingerprint, Watch};
use super::{ColumnWidths, LayoutPrefs, WindowPrefs};
use crate::theme::{self, LaneTheme};

/// How often the files are looked at.
const WATCH_EVERY: Duration = Duration::from_millis(750);

/// `ui.graph_color_theme` value that follows the active theme's own lane palette.
const FOLLOW_THEME: &str = "theme";

/// Command-line choices that override the stored settings for this run only.
#[derive(Default, Clone, Debug, PartialEq)]
pub struct Overrides {
    pub theme: Option<String>,
    pub lane_theme: Option<String>,
    pub lang: Option<Lang>,
}

/// Holds what was read, where it came from and what this process asked for. With no config directory
/// (none could be found) the settings live in memory only.
pub struct PrefsStore {
    paths: Option<ConfigPaths>,
    identity: Identity,
    /// The identity is this installation's own (not a stand-in): colours are then written the way
    /// every frontend writes them (the primary to `desktop.toml`, another installation to its entry).
    real_identity: bool,
    overrides: Overrides,
    loaded: Loaded,
    dates: DateSettings,
    watch: Watch,
}

impl Global for PrefsStore {}

impl PrefsStore {
    /// Reads the files of `paths` (the defaults without any).
    pub fn open(paths: Option<ConfigPaths>, overrides: Overrides) -> Self {
        // A config folder named by `GITPOULPI_CONFIG_DIR` (or a test) has no installation id: the
        // stand-in identity keeps the real configuration out of it.
        let real_identity = paths.as_ref().is_some_and(|p| p.install_id().is_some());
        let identity = if real_identity {
            installation::current().identity.clone()
        } else {
            anonymous()
        };
        let loaded = match &paths {
            Some(p) => Loaded::read(p, &identity),
            None => Loaded::defaults(),
        };
        let mut watch = Watch::default();
        if let Some(p) = &paths {
            watch.rebase(Fingerprint::of(p));
        }
        let dates = dates_of(&loaded.desktop.ui);
        Self {
            paths,
            identity,
            real_identity,
            overrides,
            loaded,
            dates,
            watch,
        }
    }

    pub fn get(cx: &App) -> &PrefsStore {
        cx.global::<PrefsStore>()
    }

    pub fn paths(&self) -> Option<&ConfigPaths> {
        self.paths.as_ref()
    }

    /// The whole of `desktop.toml` (the environment applied).
    pub fn desktop(&self) -> &DesktopConfig {
        &self.loaded.desktop
    }

    /// The desktop's `[ui]`: every per-interface setting the desktop reads.
    pub fn ui(&self) -> &DesktopUiConfig {
        &self.loaded.desktop.ui
    }

    /// `config.toml`: the settings both interfaces share (log loading, editor, `ui.open_in`, ...).
    pub fn common(&self) -> &Config {
        &self.loaded.common
    }

    /// The date patterns of `desktop.toml`, tokenized once.
    pub fn dates(&self) -> &DateSettings {
        &self.dates
    }

    /// Recent repositories, newest first.
    pub fn recent(&self) -> &[String] {
        &self.loaded.recent
    }

    pub fn layout(&self) -> LayoutPrefs {
        LayoutPrefs::of(self.ui())
    }

    pub fn window(&self) -> WindowPrefs {
        WindowPrefs::of(self.ui())
    }

    /// The Log's column widths for a repository: its own, else the shared default.
    pub fn columns_for(&self, repo: &str) -> ColumnWidths {
        self.desktop().column_widths_for(repo).into()
    }

    /// The config the keymap is built from: the common one (its sequences) with the desktop's own
    /// key bindings in place of the terminal UI's.
    pub fn keymap_config(&self) -> Config {
        let mut config = self.common().clone();
        config.keybindings = self.desktop().keybindings.clone();
        config
    }

    /// The slug of the theme in force: the command line, else this installation's colours.
    pub fn theme_slug(&self) -> String {
        let stored = self.loaded.look.get("theme").map_or("", String::as_str);
        let slug = self.overrides.theme.as_deref().unwrap_or(stored);
        match gitpoulpi_config::theme::canonical_slug(slug) {
            "" | gitpoulpi_config::THEME_SYSTEM => theme::DEFAULT_THEME.to_string(),
            slug => slug.to_string(),
        }
    }

    /// The graph's colour palette: `None` follows the theme's own.
    pub fn lane_theme(&self) -> Option<String> {
        let stored = self
            .loaded
            .look
            .get("graph_color_theme")
            .map_or("", String::as_str);
        let slug = self.overrides.lane_theme.as_deref().unwrap_or(stored);
        match gitpoulpi_config::theme::canonical_slug(slug) {
            "" | FOLLOW_THEME => None,
            slug => Some(slug.to_string()),
        }
    }

    /// The interface language: the command line, then `GITPOULPI_LANG`, then `ui.language` (`auto`
    /// follows the system).
    pub fn language(&self) -> Lang {
        if let Some(lang) = self.overrides.lang {
            return lang;
        }
        let setting = match self.ui().language {
            gitpoulpi_config::UiLanguage::Auto => None,
            explicit => Lang::parse(explicit.as_str()),
        };
        let env = std::env::var(gitpoulpi_i18n::ENV_VAR).ok();
        gitpoulpi_i18n::resolve(env.as_deref(), setting)
    }

    // ---- Applying what was read ---------------------------------------------------------------

    /// Publishes the language, the theme, the graph palette and the key bindings. Called at start-up
    /// and again when a file changed. Returns the key bindings that could not be honoured.
    pub fn install(cx: &mut App) -> Vec<gitpoulpi_app::keymap::KeymapIssue> {
        let store = Self::get(cx);
        let (lang, slug, lane) = (store.language(), store.theme_slug(), store.lane_theme());
        let (translate, english) = (store.ui().translate_git_terms, store.ui().git_terms_english);
        let config = store.keymap_config();
        gitpoulpi_i18n::set(lang);
        gitpoulpi_i18n::set_translate_git_terms(translate);
        gitpoulpi_i18n::set_english_git_terms(english);
        cx.set_global(LaneTheme(lane));
        theme::apply(&theme::find_preset(&slug), cx);
        crate::commands::keys::install_keymap(&config)
    }

    /// Takes a fresh read: what changed is applied and the window told.
    pub fn apply(cx: &mut App, loaded: Loaded) {
        let (before_ui, before_keys, before_look) = {
            let s = Self::get(cx);
            (
                s.ui().clone(),
                s.desktop().keybindings.clone(),
                s.loaded.look.clone(),
            )
        };
        {
            let store = cx.global_mut::<PrefsStore>();
            store.dates = dates_of(&loaded.desktop.ui);
            store.loaded = loaded;
        }
        let store = Self::get(cx);
        let appearance = store.loaded.look != before_look
            || store.ui().language != before_ui.language
            || store.ui().translate_git_terms != before_ui.translate_git_terms
            || store.ui().git_terms_english != before_ui.git_terms_english;
        let keys = store.desktop().keybindings != before_keys;
        if appearance || keys {
            let issues = Self::install(cx);
            if keys {
                if let Some(text) = gitpoulpi_app::keymap::issues_message(&issues) {
                    crate::commands::status::set(cx, text);
                }
            }
        }
        if let Some(ws) = crate::app::workspace_of(cx) {
            ws.update(cx, |ws, cx| ws.settings_changed(cx));
        }
        cx.refresh_windows();
    }

    /// Looks at the files once: a change starts a re-read off the UI thread, applied when it is done.
    pub fn check(cx: &mut App) {
        if !cx.has_global::<PrefsStore>() {
            return;
        }
        let store = cx.global_mut::<PrefsStore>();
        let Some(paths) = store.paths.clone() else {
            return;
        };
        if !store.watch.changed(Fingerprint::of(&paths)) {
            return;
        }
        let identity = store.identity.clone();
        cx.spawn(async move |cx| {
            let loaded = cx
                .background_executor()
                .spawn(async move { Loaded::read(&paths, &identity) })
                .await;
            cx.update(|cx| PrefsStore::apply(cx, loaded));
        })
        .detach();
    }

    /// Starts the timer that calls [`PrefsStore::check`]. Call once.
    pub fn watch(cx: &mut App) {
        cx.spawn(async move |cx| loop {
            cx.background_executor().timer(WATCH_EVERY).await;
            cx.update(PrefsStore::check);
        })
        .detach();
    }

    // ---- Writing -------------------------------------------------------------------------------

    /// Applies `mem` to what is held and `disk` to the files; the watch learns the new fingerprint so
    /// this process's own write is not taken for an outside edit. A failed write is reported once on
    /// stderr and never interrupts the user.
    fn write(
        cx: &mut App,
        mem: impl FnOnce(&mut Loaded),
        disk: impl FnOnce(&ConfigPaths, &Identity, bool) -> gitpoulpi_config::Result<()>,
    ) {
        let store = cx.global_mut::<PrefsStore>();
        mem(&mut store.loaded);
        store.dates = dates_of(&store.loaded.desktop.ui);
        if let Some(paths) = store.paths.clone() {
            if let Err(e) = disk(&paths, &store.identity, store.real_identity) {
                eprintln!("desktop settings not saved: {e}");
            }
            store.watch.rebase(Fingerprint::of(&paths));
        }
    }

    /// Remembers a colour key: the primary installation writes `desktop.toml`, another one its own
    /// entry (so one installation's pick never recolours another).
    fn write_look(cx: &mut App, key: &'static str, value: String) {
        let stored = value.clone();
        Self::write(
            cx,
            move |l| {
                l.look.insert(key.to_string(), stored);
            },
            move |paths, identity, real| {
                if real {
                    installation::set_desktop_look_in_file(paths, identity, key, &value)?;
                } else {
                    DesktopConfig::set_in_file(paths, &format!("ui.{key}"), &value)?;
                }
                Ok(())
            },
        );
    }

    pub fn set_theme(cx: &mut App, slug: &str) {
        // An explicit pick ends the command line's say for the rest of the run.
        cx.global_mut::<PrefsStore>().overrides.theme = None;
        Self::write_look(cx, "theme", slug.to_string());
    }

    /// `None` follows the theme's own graph palette.
    pub fn set_lane_theme(cx: &mut App, slug: Option<&str>) {
        cx.global_mut::<PrefsStore>().overrides.lane_theme = None;
        Self::write_look(
            cx,
            "graph_color_theme",
            slug.unwrap_or(FOLLOW_THEME).to_string(),
        );
    }

    pub fn set_language(cx: &mut App, lang: Lang) {
        cx.global_mut::<PrefsStore>().overrides.lang = None;
        let code = lang.code().to_string();
        let stored = code.clone();
        Self::write(
            cx,
            move |l| {
                if let Some(ui) = gitpoulpi_config::UiLanguage::ALL
                    .iter()
                    .find(|u| u.as_str() == stored)
                {
                    l.desktop.ui.language = *ui;
                }
            },
            move |paths, _, _| {
                DesktopConfig::set_in_file(paths, "ui.language", &code)?;
                Ok(())
            },
        );
    }

    /// The side panes: which are shown and how wide (the widths are kept inside their limits).
    pub fn set_panes(cx: &mut App, left_visible: bool, right_visible: bool, left: f32, right: f32) {
        let (lw, rw) = (left.round().max(0.0) as u32, right.round().max(0.0) as u32);
        Self::write(
            cx,
            move |l| {
                let ui = &mut l.desktop.ui;
                ui.left_pane_visible = left_visible;
                ui.right_pane_visible = right_visible;
                ui.left_pane_width = lw;
                ui.right_pane_width = rw;
                ui.sanitize();
            },
            move |paths, _, _| DesktopConfig::set_panes(paths, left_visible, right_visible, lw, rw),
        );
    }

    /// The main window's size, position and whether it is maximized (kept inside the supported range).
    pub fn set_window(cx: &mut App, window: WindowPrefs) {
        let size = (
            window.width.round().max(0.0) as u32,
            window.height.round().max(0.0) as u32,
        );
        let position = window
            .position
            .map(|(x, y)| (x.round() as i32, y.round() as i32));
        let maximized = window.maximized;
        Self::write(
            cx,
            move |l| {
                let ui = &mut l.desktop.ui;
                ui.window_width = size.0;
                ui.window_height = size.1;
                ui.window_x = position.map(|p| p.0);
                ui.window_y = position.map(|p| p.1);
                ui.window_maximized = maximized;
                ui.sanitize();
            },
            move |paths, _, _| DesktopConfig::set_window(paths, size, position, maximized),
        );
    }

    /// Remembers Log column widths: for `repo`, or for every repository without its own with `None`.
    pub fn set_columns(cx: &mut App, repo: Option<&str>, widths: ColumnWidths) {
        let widths: DesktopColumnWidths = widths.into();
        let key = repo.map(str::to_string);
        let stored = key.clone();
        Self::write(
            cx,
            move |l| match stored {
                Some(repo) => {
                    l.desktop.repos.entry(repo).or_default().column_widths_px = Some(widths)
                }
                None => l.desktop.ui.column_widths_px = widths,
            },
            move |paths, _, _| DesktopConfig::set_column_widths(paths, key.as_deref(), widths),
        );
    }

    /// Forgets a repository's own widths: it follows the shared default again.
    pub fn reset_columns_for(cx: &mut App, repo: &str) {
        let (a, b) = (repo.to_string(), repo.to_string());
        Self::write(
            cx,
            move |l| {
                if let Some(r) = l.desktop.repos.get_mut(&a) {
                    r.column_widths_px = None;
                }
            },
            move |paths, _, _| DesktopConfig::reset_column_widths(paths, &b),
        );
    }

    /// Puts `repo` at the front of the recent list (the one the terminal UI and the CLI share).
    pub fn remember_recent(cx: &mut App, repo: &str) {
        let path = std::path::PathBuf::from(repo);
        let name = path
            .file_name()
            .map_or_else(|| repo.to_string(), |n| n.to_string_lossy().into_owned());
        let stored = repo.to_string();
        Self::write(
            cx,
            move |l| {
                l.recent.retain(|r| *r != stored);
                l.recent.insert(0, stored);
                l.recent.truncate(gitpoulpi_session::MAX_RECENT);
            },
            move |paths, _, _| {
                let file = RecentRepos::file_in(paths.config_dir());
                let mut recents = RecentRepos::load(&file);
                recents.record(&path, &name, None, gitpoulpi_session::now_seconds());
                recents
                    .save(&file)
                    .map_err(|e| gitpoulpi_config::Error::Io(std::io::Error::other(e.to_string())))
            },
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_missing_config_directory_gives_the_defaults() {
        let store = PrefsStore::open(None, Overrides::default());
        assert_eq!(store.desktop(), &DesktopConfig::default());
        assert_eq!(store.theme_slug(), theme::DEFAULT_THEME);
        assert_eq!(store.lane_theme(), None);
        assert!(store.recent().is_empty());
    }

    #[test]
    fn the_command_line_outranks_the_file() {
        let dir = tempfile::tempdir().unwrap();
        let paths = ConfigPaths::for_dir(dir.path());
        std::fs::write(paths.desktop_file(), "[ui]\ntheme = \"nord\"\n").unwrap();
        let file_only = PrefsStore::open(Some(paths.clone()), Overrides::default());
        assert_eq!(file_only.theme_slug(), "nord");
        let overridden = PrefsStore::open(
            Some(paths),
            Overrides {
                theme: Some("dusk".into()),
                lang: Some(Lang::Fr),
                ..Overrides::default()
            },
        );
        assert_eq!(overridden.theme_slug(), "dusk");
        assert_eq!(overridden.language(), Lang::Fr);
    }

    #[test]
    fn the_common_config_and_the_keymap_config_are_kept_apart() {
        let dir = tempfile::tempdir().unwrap();
        let paths = ConfigPaths::for_dir(dir.path());
        std::fs::write(
            paths.desktop_file(),
            "[keybindings.global]\npalette = \"Ctrl+K\"\n",
        )
        .unwrap();
        let store = PrefsStore::open(Some(paths), Overrides::default());
        assert!(store.keymap_config().keybindings != store.common().keybindings);
    }
}
