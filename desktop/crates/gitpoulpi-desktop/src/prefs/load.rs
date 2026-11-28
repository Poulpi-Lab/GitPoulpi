//! Reading the settings files: a pure function of the config directory, so it can run on a
//! background thread and be tested against a temporary directory.

use std::collections::BTreeMap;

use gitpoulpi_app::dates::DateSettings;
use gitpoulpi_config::desktop_ui_file::DesktopSettings;
use gitpoulpi_config::installation::{effective_desktop_look, Identity};
use gitpoulpi_config::{Config, ConfigPaths, DesktopConfig, DesktopUiConfig, UiConfig};
use gitpoulpi_session::RecentRepos;

/// Everything one read of the files yields.
#[derive(Clone, Debug)]
pub struct Loaded {
    /// `desktop.toml` with the `GITPOULPI_DESKTOP_UI_*` environment applied.
    pub desktop: DesktopConfig,
    /// `config.toml`: what both interfaces share.
    pub common: Config,
    /// The colour keys this installation shows (`theme`, `graph_color_theme`, ...).
    pub look: BTreeMap<String, String>,
    /// Recent repositories, newest first.
    pub recent: Vec<String>,
}

impl Loaded {
    /// Reads the files in `paths`. A missing file is the defaults; a file that does not parse leaves
    /// the defaults too (the window must start whatever a hand edit did), an environment variable
    /// that is refused is ignored.
    pub fn read(paths: &ConfigPaths, identity: &Identity) -> Self {
        Self::read_with(paths, identity, |name| std::env::var(name).ok())
    }

    /// [`Loaded::read`] with the environment as a lookup (a test never sets a real variable: the
    /// process environment is shared by every test running next to it).
    pub fn read_with(
        paths: &ConfigPaths,
        identity: &Identity,
        env: impl Fn(&str) -> Option<String>,
    ) -> Self {
        let desktop = DesktopSettings::resolve_from(&paths.desktop_file(), env)
            .map(|s| s.config().clone())
            .unwrap_or_else(|_| DesktopConfig::load_or_default(paths));
        let common = Config::load(&paths.config_file()).unwrap_or_default();
        let look = look_of(&common, &desktop.ui, identity);
        let recent = RecentRepos::load(&RecentRepos::file_in(paths.config_dir()))
            .iter()
            .map(|r| r.path.to_string_lossy().into_owned())
            .collect();
        Self {
            desktop,
            common,
            look,
            recent,
        }
    }

    /// What nothing was read from: the built-in defaults.
    pub fn defaults() -> Self {
        let desktop = DesktopConfig::default();
        let look = look_of(&Config::default(), &desktop.ui, &anonymous());
        Self {
            desktop,
            common: Config::default(),
            look,
            recent: Vec::new(),
        }
    }
}

/// The installation a window opened without a real one (a test, a config folder given by hand) is.
pub fn anonymous() -> Identity {
    Identity {
        id: "desktop".to_string(),
        kind: gitpoulpi_config::Kind::Pinned,
        root: String::new(),
        channel: None,
    }
}

/// This installation's colours. With no primary installation recorded yet, this one is it (the first
/// frontend to write is how a primary is chosen), so a lone window follows `desktop.toml`.
pub fn look_of(
    common: &Config,
    ui: &DesktopUiConfig,
    identity: &Identity,
) -> BTreeMap<String, String> {
    let mut config = common.clone();
    if config.installation.primary.is_none() {
        config.installation.primary = Some(identity.id.clone());
    }
    effective_desktop_look(&config, ui, &identity.id)
}

/// The date patterns of the desktop's `[ui]`, tokenized once.
pub fn dates_of(ui: &DesktopUiConfig) -> DateSettings {
    let shared = UiConfig {
        date_format: ui.date_format.clone(),
        date_time_format: ui.date_time_format.clone(),
        date_verbose_format: ui.date_verbose_format.clone(),
        date_locale: ui.date_locale.clone(),
        ..UiConfig::default()
    };
    DateSettings::from_config(&shared)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_missing_file_is_the_defaults_and_is_not_created() {
        let dir = tempfile::tempdir().unwrap();
        let paths = ConfigPaths::for_dir(dir.path());
        let loaded = Loaded::read(&paths, &anonymous());
        assert_eq!(loaded.desktop, DesktopConfig::default());
        assert!(loaded.recent.is_empty());
        assert!(!paths.desktop_file().exists());
    }

    #[test]
    fn a_file_that_does_not_parse_leaves_the_defaults() {
        let dir = tempfile::tempdir().unwrap();
        let paths = ConfigPaths::for_dir(dir.path());
        std::fs::write(paths.desktop_file(), "this is [not toml").unwrap();
        let loaded = Loaded::read(&paths, &anonymous());
        assert_eq!(loaded.desktop, DesktopConfig::default());
    }

    #[test]
    fn the_lone_installation_follows_desktop_toml() {
        let dir = tempfile::tempdir().unwrap();
        let paths = ConfigPaths::for_dir(dir.path());
        std::fs::write(paths.desktop_file(), "[ui]\ntheme = \"nord\"\n").unwrap();
        let loaded = Loaded::read(&paths, &anonymous());
        assert_eq!(loaded.look["theme"], "nord");
    }

    #[test]
    fn the_environment_outranks_the_file_for_the_live_keys() {
        let dir = tempfile::tempdir().unwrap();
        let paths = ConfigPaths::for_dir(dir.path());
        std::fs::write(paths.desktop_file(), "[ui]\ntheme = \"nord\"\n").unwrap();
        let env = |name: &str| (name == "GITPOULPI_DESKTOP_UI_THEME").then(|| "dusk".to_string());
        let loaded = Loaded::read_with(&paths, &anonymous(), env);
        assert_eq!(loaded.look["theme"], "dusk");
        // A refused value is ignored: the window still starts, from the file.
        let bad = |name: &str| {
            (name == "GITPOULPI_DESKTOP_UI_DIFF_LAYOUT").then(|| "sideways".to_string())
        };
        let loaded = Loaded::read_with(&paths, &anonymous(), bad);
        assert_eq!(loaded.look["theme"], "nord");
    }

    #[test]
    fn the_date_patterns_come_from_the_desktop_ui() {
        let ui = DesktopUiConfig {
            date_time_format: "DD/MM/YYYY".into(),
            ..DesktopUiConfig::default()
        };
        assert_eq!(dates_of(&ui).format_time(86_400 * 365), "01/01/1971");
    }
}
