//! The config schema.
//!
//! Every field here parses and round-trips, but only the keys listed in
//! `crate::LIVE_KEYS` are read by any component -- see that constant and
//! `docs/user/config-reference.md`, which marks the rest. Adding a field
//! here is not the same as shipping a setting.
//!
//! The Preferences screen in the TUI (`crates/gitpoulpi-tui/src/
//! preferences.rs`) renders these sections directly, which makes that
//! discipline visible to the user rather than only to whoever reads this
//! file: a field nothing reads is drawn with a `not wired yet` note, and
//! the note is generated from `LIVE_KEYS`, so it cannot drift.

use serde::{Deserialize, Serialize};

/// Declares a closed-set config enum: each variant is written once with
/// the name it has in `config.toml`, and the macro derives from that the
/// serde rename, `ALL` (declaration order) and `as_str`, so the three
/// cannot disagree. `tests/schema_roundtrip.rs` checks the round trip.
macro_rules! config_enum {
    (
        $(#[$meta:meta])*
        pub enum $name:ident {
            $( $(#[$vmeta:meta])* $variant:ident => $text:literal ),+ $(,)?
        }
    ) => {
        $(#[$meta])*
        pub enum $name {
            $( $(#[$vmeta])* #[serde(rename = $text)] $variant ),+
        }

        impl $name {
            pub const ALL: &'static [$name] = &[$($name::$variant),+];

            pub fn as_str(self) -> &'static str {
                match self {
                    $($name::$variant => $text),+
                }
            }
        }
    };
}

mod editor;
mod general;
mod git;
mod log;
mod quantity;
mod rewrite;
mod search;
mod ssh;
mod ui;
mod updates;

pub use editor::*;
pub use general::*;
pub use git::*;
pub use log::*;
pub use quantity::*;
pub use rewrite::*;
pub use search::*;
pub use ssh::*;
pub use ui::*;
pub use updates::*;

/// The pre-`[[integrations]]` shape, kept so an existing `config.toml`
/// still parses. Nothing reads it; the Preferences screen writes
/// `[[integrations]]`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HostingInstance {
    pub provider: String,
    pub name: Option<String>,
    pub base_url: String,
    pub auth: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct McpConfig {
    /// `false`: `gitpoulpi mcp serve` and `gitpoulpi-mcp` refuse to start.
    /// On by default (the server only runs when an agent starts it).
    pub enabled: bool,
    /// Default listen address of `--http` (without an explicit address),
    /// loopback only unless `--allow-remote`. Empty or the legacy
    /// `"stdio"` mean the built-in `127.0.0.1:8808`.
    pub bind: String,
    /// `false` (the default): MCP tools that change the repository are
    /// refused; reads, `ui_*` and proposals still work. The user opts in.
    pub allow_mutations: bool,
    pub mutation_allowlist: Vec<String>,
    /// Whether the `ui_*` tools (an agent showing the user a commit, a
    /// diff, a file...) may run at all. They are view-only: they never
    /// change the repository, so `allow_mutations = false` does not block
    /// them. `false` refuses every one of them except `ui_state` and
    /// `ui_viewers`.
    pub allow_ui: bool,
    /// How far an agent may move the user's window by itself: `ask` (the
    /// default: the window shows a chip the user confirms), `auto` or
    /// `off`. Carried with every view intent so the TUI can honour it;
    /// `gitpoulpi-mcp --ui-control` overrides it per agent setup.
    pub ui_control: gitpoulpi_core::domain::UiControl,
    /// Whether an agent may *propose* a change for the user to confirm
    /// (`ui propose reword|commit-message|branch|tag`, the MCP
    /// `ui_propose_*` tools): a dialog opens pre-filled and nothing happens
    /// until the user presses its own confirm. Off by default - opt-in,
    /// because it ends in a repository change. `allow_ui` and
    /// `ui_control` still apply on top of it.
    pub allow_ui_propose: bool,
    /// Whether the MCP tool `rewrite_purge` exists at all. Off by default
    /// and only in `config.toml` (no Settings row): a purge deletes the old
    /// history of a rewrite, the undo of it and its reflogs for good, so an
    /// agent does not even see the tool unless the operator opts in. The
    /// CLI and the TUI are not gated by it.
    pub allow_history_purge: bool,
}

/// The port of the built-in MCP listen address.
pub const MCP_DEFAULT_PORT: u16 = 8808;
/// The host of the built-in MCP listen address.
pub const MCP_DEFAULT_HOST: &str = "127.0.0.1";

impl McpConfig {
    /// The host and port parts of `bind`. Empty and the legacy `"stdio"`
    /// mean the built-in address. `None` for the port when `bind` has no
    /// readable `:port` (the host then keeps whatever precedes the last
    /// colon, or the whole text when there is none).
    pub fn bind_parts(&self) -> (String, Option<u16>) {
        let bind = self.bind.trim();
        if bind.is_empty() || bind == "stdio" {
            return (MCP_DEFAULT_HOST.into(), Some(MCP_DEFAULT_PORT));
        }
        match bind.rsplit_once(':') {
            Some((host, port)) => (
                if host.is_empty() {
                    MCP_DEFAULT_HOST.into()
                } else {
                    host.into()
                },
                port.parse::<u16>().ok().filter(|p| *p != 0),
            ),
            None => (bind.into(), None),
        }
    }

    /// `bind` with its port replaced, the host kept.
    pub fn bind_with_port(&self, port: u16) -> String {
        format!("{}:{port}", self.bind_parts().0)
    }
}

impl Default for McpConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            bind: "127.0.0.1:8808".into(),
            allow_mutations: false,
            mutation_allowlist: Vec::new(),
            allow_ui: true,
            ui_control: gitpoulpi_core::domain::UiControl::Ask,
            allow_ui_propose: false,
            allow_history_purge: false,
        }
    }
}

/// Where repositories live on this machine.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct ReposConfig {
    /// Live: the folder the Clone dialog's "Where to clone to" starts
    /// on. Empty means the per-OS default ([`default_clone_root`]).
    pub clone_root: String,
    /// Live: the folder Browse (the folder picker, reached from
    /// Repository Management) opens on by default. Empty means "no
    /// override" - Browse keeps opening on the last folder chosen, or
    /// on the home folder before anything has been chosen yet. See
    /// [`ReposConfig::browse_root_override`].
    pub browse_root: String,
    /// Live: the folder picker's "Show hidden folders" checkbox -
    /// remembered across pickers and restarts.
    pub show_hidden: bool,
    /// Live: the folder picker's "Show sibling folders" checkbox - the
    /// left tree also lists the other folders of every open level.
    pub show_siblings: bool,
    /// Live: the source the Clone dialog opens on. `last` (the default,
    /// also an empty value) reopens the source it showed when it was last
    /// closed, `url` is always "Clone with URL", anything else is an
    /// integration profile id (a profile that cannot be a source: URL).
    pub clone_start_source: String,
}

impl Default for ReposConfig {
    fn default() -> Self {
        Self {
            clone_root: String::new(),
            browse_root: String::new(),
            show_hidden: false,
            show_siblings: false,
            clone_start_source: CLONE_START_LAST.into(),
        }
    }
}

/// `repos.clone_start_source`: reopen on the last source shown.
pub const CLONE_START_LAST: &str = "last";
/// `repos.clone_start_source`: always start on "Clone with URL".
pub const CLONE_START_URL: &str = "url";

impl ReposConfig {
    /// The folder a clone lands under: the configured one, or the per-OS
    /// default when the key is empty. `None` only when neither is known
    /// (no configured value and no home directory).
    pub fn clone_root_or_default(&self) -> Option<std::path::PathBuf> {
        let configured = self.clone_root.trim();
        if configured.is_empty() {
            default_clone_root(home_dir().as_deref())
        } else {
            Some(std::path::PathBuf::from(configured))
        }
    }

    /// The configured override for where Browse opens, if one is set.
    ///
    /// Unlike [`Self::clone_root_or_default`] this has no per-OS
    /// fallback of its own: an empty key means Browse's caller should
    /// fall through to *its* own defaults (the last folder chosen, then
    /// the home folder) rather than to a fixed folder nobody asked for.
    pub fn browse_root_override(&self) -> Option<std::path::PathBuf> {
        let configured = self.browse_root.trim();
        (!configured.is_empty()).then(|| std::path::PathBuf::from(configured))
    }
}

/// The per-OS default for `repos.clone_root`: `%USERPROFILE%\source\repos`
/// on Windows (the Visual Studio / GitHub Desktop convention), `~/src`
/// elsewhere. Pure: the home directory is passed in.
pub fn default_clone_root(home: Option<&std::path::Path>) -> Option<std::path::PathBuf> {
    let home = home?;
    Some(if cfg!(windows) {
        home.join("source").join("repos")
    } else {
        home.join("src")
    })
}

/// The user's home directory: `USERPROFILE` first on Windows (`HOME`
/// there may be a POSIX-style path set by a Unix shell), `HOME` first
/// elsewhere.
pub fn home_dir() -> Option<std::path::PathBuf> {
    let order: [&str; 2] = if cfg!(windows) {
        ["USERPROFILE", "HOME"]
    } else {
        ["HOME", "USERPROFILE"]
    };
    order
        .iter()
        .filter_map(std::env::var_os)
        .find(|v| !v.is_empty())
        .map(std::path::PathBuf::from)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct SessionConfig {
    /// Where per-repository data lives: `in_git_dir` (default,
    /// `<git dir>/gitpoulpi`), `in_worktree` (`<repo>/.gitpoulpi`) or
    /// `user_data` (`<OS data dir>/repos/<name>-<id>`). The old value
    /// `".gitpoulpi"` still reads as `in_worktree`. See
    /// [`crate::RepoDataLocation`].
    pub per_repo_state_dir: String,
}

impl Default for SessionConfig {
    fn default() -> Self {
        Self {
            per_repo_state_dir: crate::DEFAULT_REPO_DATA_LOCATION.into(),
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn undo_depth_defaults_to_a_hundred_and_zero_means_unlimited() {
        let mut general = GeneralConfig::default();
        assert_eq!(general.undo_depth, 100);
        assert_eq!(general.undo_depth_limit(), 100);
        general.undo_depth = 0;
        assert_eq!(general.undo_depth_limit(), usize::MAX);
        general.undo_depth = 7;
        assert_eq!(general.undo_depth_limit(), 7);
        let parsed: crate::Config = toml::from_str("[general]\nundo_depth = 0\n").unwrap();
        assert_eq!(parsed.general.undo_depth, 0);
        assert!(general.undo_enabled, "undo is on by default");
        let off: crate::Config = toml::from_str("[general]\nundo_enabled = false\n").unwrap();
        assert!(!off.general.undo_enabled);
    }

    use super::*;

    /// Keys of features that were removed must not make an old config fail.
    #[test]
    fn a_stored_secrets_file_key_still_loads_and_is_ignored() {
        let parsed: crate::Config = toml::from_str(
            "[credentials]\nhttps_mode = \"credential_helper\"\nsecrets_file = \"~/x.age\"\n",
        )
        .unwrap();
        assert_eq!(parsed.credentials.https_mode, HttpsMode::CredentialHelper);
    }

    /// The encrypted-file modes left with the secrets file; configs that
    /// name them keep loading, as the credential helper.
    #[test]
    fn the_removed_https_modes_load_as_the_credential_helper() {
        for old in ["encrypted_file", "credential_helper_then_encrypted_file"] {
            let parsed: crate::Config =
                toml::from_str(&format!("[credentials]\nhttps_mode = \"{old}\"\n")).unwrap();
            assert_eq!(parsed.credentials.https_mode, HttpsMode::CredentialHelper);
        }
        for (name, mode) in [("prompt", HttpsMode::Prompt), ("none", HttpsMode::None)] {
            let parsed: crate::Config =
                toml::from_str(&format!("[credentials]\nhttps_mode = \"{name}\"\n")).unwrap();
            assert_eq!(parsed.credentials.https_mode, mode);
            assert_eq!(mode.as_str(), name);
        }
    }

    /// `ssh.use_agent` was removed; a config that still has it must load.
    #[test]
    fn a_stored_ssh_use_agent_key_still_loads_and_is_ignored() {
        let parsed: crate::Config =
            toml::from_str("[ssh]\nuse_agent = false\nkey_type = \"rsa\"\n").unwrap();
        assert_eq!(parsed.ssh.key_type, "rsa");
    }

    /// The setting grew from `off|subtle|strong` (and, in some hands, a
    /// boolean) to `off|lane|row_tint`; every spelling ever written keeps
    /// loading.
    #[test]
    fn the_lane_background_reads_every_spelling_it_ever_had() {
        #[derive(Deserialize)]
        struct Probe {
            v: GraphLaneBackground,
        }
        let read = |raw: &str| toml::from_str::<Probe>(&format!("v = {raw}")).unwrap().v;
        assert_eq!(read("true"), GraphLaneBackground::RowTint);
        assert_eq!(read("false"), GraphLaneBackground::Off);
        assert_eq!(read("\"off\""), GraphLaneBackground::Off);
        assert_eq!(read("\"lane\""), GraphLaneBackground::RowTint);
        assert_eq!(read("\"subtle\""), GraphLaneBackground::RowTint);
        assert_eq!(read("\"strong\""), GraphLaneBackground::RowTint);
        assert_eq!(read("\"row_tint\""), GraphLaneBackground::RowTint);
        assert!(toml::from_str::<Probe>("v = \"loud\"").is_err());
        for each in GraphLaneBackground::ALL {
            assert_eq!(read(&format!("\"{}\"", each.as_str())), *each);
        }
    }

    /// The historical spelling has to keep parsing: a config file written
    /// before `ui.theme` became an enum is still on the user's disk, and
    /// an upgrade that turns it into a parse error takes the whole app
    /// down rather than one setting.
    /// An empty `repos.clone_root` falls back to the per-OS default,
    /// and a configured one wins.
    #[test]
    fn the_clone_root_defaults_per_os_and_a_configured_one_wins() {
        let home = std::path::Path::new("/home/someone");
        let default = default_clone_root(Some(home)).unwrap();
        if cfg!(windows) {
            assert_eq!(default, home.join("source").join("repos"));
        } else {
            assert_eq!(default, home.join("src"));
        }
        assert_eq!(default_clone_root(None), None);
        let configured = ReposConfig {
            clone_root: "/work/clones".into(),
            ..ReposConfig::default()
        };
        assert_eq!(
            configured.clone_root_or_default(),
            Some(std::path::PathBuf::from("/work/clones"))
        );
        let parsed: ReposConfig = toml::from_str("").unwrap();
        assert_eq!(parsed.clone_root, "");
    }

    /// `repos.browse_root` has no per-OS fallback of its own - unset, it
    /// must say `None` rather than guessing at a folder, so its caller's
    /// own "last folder, then home" fallback still runs.
    #[test]
    fn an_unset_browse_root_is_none_and_a_configured_one_wins() {
        let unset = ReposConfig::default();
        assert_eq!(unset.browse_root_override(), None);
        let configured = ReposConfig {
            browse_root: "/work/projects".into(),
            ..ReposConfig::default()
        };
        assert_eq!(
            configured.browse_root_override(),
            Some(std::path::PathBuf::from("/work/projects"))
        );
        // Whitespace-only is unset too, matching `clone_root`'s rule.
        let blank = ReposConfig {
            browse_root: "   ".into(),
            ..ReposConfig::default()
        };
        assert_eq!(blank.browse_root_override(), None);
    }

    #[test]
    fn hosting_refresh_defaults_to_five_minutes_and_zero_disables() {
        let g = GeneralConfig::default();
        assert_eq!(
            g.hosting_refresh_interval(),
            Some(std::time::Duration::from_secs(300))
        );
        let off: GeneralConfig = toml::from_str(
            "hosting_refresh_minutes = 0
",
        )
        .unwrap();
        assert_eq!(off.hosting_refresh_interval(), None);
        let huge: GeneralConfig = toml::from_str(
            "hosting_refresh_minutes = 9999
",
        )
        .unwrap();
        assert_eq!(
            huge.hosting_refresh_interval(),
            Some(std::time::Duration::from_secs(120 * 60))
        );
    }

    #[test]
    fn the_old_theme_spelling_still_parses() {
        let ui: UiConfig = toml::from_str("theme = \"default\"\n").unwrap();
        assert_eq!(ui.theme_slug(), "system");
    }

    #[test]
    fn ref_connector_defaults_to_on() {
        assert!(UiConfig::default().ref_connector);
        let empty: UiConfig = toml::from_str("").unwrap();
        assert!(empty.ref_connector);
        let off: UiConfig = toml::from_str("ref_connector = false\n").unwrap();
        assert!(!off.ref_connector);
    }

    /// A setting that discloses a hash of the user's git history's author
    /// emails to a third party must not turn itself on.
    #[test]
    fn worktree_tab_grouping_and_the_ghost_ref_default_to_on_and_parse_off() {
        assert!(GeneralConfig::default().group_worktree_tabs);
        assert!(UiConfig::default().graph_ghost_ref);
        let g: GeneralConfig = toml::from_str("group_worktree_tabs = false\n").unwrap();
        assert!(!g.group_worktree_tabs);
        let u: UiConfig = toml::from_str("graph_ghost_ref = false\n").unwrap();
        assert!(!u.graph_ghost_ref);
        let empty: UiConfig = toml::from_str("").unwrap();
        assert!(empty.graph_ghost_ref);
    }

    /// Older commits load as you scroll by default; a fresh (empty) config
    /// gets lazy pages, and an explicit `false` still turns them off.
    #[test]
    fn lazy_loading_commits_in_the_graph_defaults_to_on() {
        assert!(GeneralConfig::default().lazy_load_commits_in_graph);
        let empty: GeneralConfig = toml::from_str("").unwrap();
        assert!(empty.lazy_load_commits_in_graph);
        let off: GeneralConfig = toml::from_str("lazy_load_commits_in_graph = false\n").unwrap();
        assert!(!off.lazy_load_commits_in_graph);
    }

    #[test]
    fn graph_show_stashes_defaults_to_on() {
        assert!(UiConfig::default().graph_show_stashes);
        let empty: UiConfig = toml::from_str("").unwrap();
        assert!(empty.graph_show_stashes);
        let off: UiConfig = toml::from_str("graph_show_stashes = false\n").unwrap();
        assert!(!off.graph_show_stashes);
    }

    #[test]
    fn show_avatars_defaults_to_off() {
        assert!(!UiConfig::default().show_avatars);
        let empty: UiConfig = toml::from_str("").unwrap();
        assert!(!empty.show_avatars);
        let on: UiConfig = toml::from_str("show_avatars = true\n").unwrap();
        assert!(on.show_avatars);
    }

    /// The migration: the legacy bool is honoured while `avatars` is
    /// absent, and an explicit `avatars` wins over it either way.
    #[test]
    fn the_legacy_show_avatars_migrates_to_author_until_avatars_is_set() {
        let placement = |toml: &str| toml::from_str::<UiConfig>(toml).unwrap().avatar_placement();
        assert_eq!(placement(""), AvatarPlacement::Off);
        assert_eq!(placement("show_avatars = true\n"), AvatarPlacement::Author);
        assert_eq!(placement("show_avatars = false\n"), AvatarPlacement::Off);
        assert_eq!(placement("avatars = \"graph\"\n"), AvatarPlacement::Graph);
        assert_eq!(
            placement("show_avatars = true\navatars = \"off\"\n"),
            AvatarPlacement::Off
        );
        assert_eq!(
            placement("show_avatars = false\navatars = \"author\"\n"),
            AvatarPlacement::Author
        );
        assert!(toml::from_str::<UiConfig>("avatars = \"sideways\"\n").is_err());
    }

    #[test]
    fn avatar_sources_and_fallback_default_and_parse() {
        let ui = UiConfig::default();
        assert_eq!(ui.avatar_sources, AvatarSources::GravatarHosting);
        assert_eq!(ui.avatar_fallback, AvatarFallback::Identicon);
        for v in AvatarSources::ALL {
            let ui: UiConfig =
                toml::from_str(&format!("avatar_sources = \"{}\"\n", v.as_str())).unwrap();
            assert_eq!(ui.avatar_sources, *v);
        }
        for v in AvatarFallback::ALL {
            let ui: UiConfig =
                toml::from_str(&format!("avatar_fallback = \"{}\"\n", v.as_str())).unwrap();
            assert_eq!(ui.avatar_fallback, *v);
        }
        assert!(toml::from_str::<UiConfig>("avatar_sources = \"x\"\n").is_err());
        let mut config = crate::Config::default();
        config.set("ui.avatar_sources", "gravatar").unwrap();
        config.set("ui.avatar_fallback", "initials").unwrap();
        config.set("ui.composer_files", "collapsed").unwrap();
        assert_eq!(config.ui.composer_files, ComposerFiles::Collapsed);
        assert_eq!(config.ui.avatar_sources, AvatarSources::Gravatar);
        assert_eq!(config.ui.avatar_fallback, AvatarFallback::Initials);
    }

    /// `ui.avatars` is settable from a shell although its default is
    /// "absent" (`null` in the JSON view `Config::set` works on).
    #[test]
    fn avatars_is_settable_by_dotted_key_and_absent_from_a_default_file() {
        let mut config = crate::Config::default();
        config.set("ui.avatars", "graph").unwrap();
        assert_eq!(config.ui.avatar_placement(), AvatarPlacement::Graph);
        assert!(config.set("ui.avatars", "sideways").is_err());
        let raw = toml::to_string(&crate::Config::default()).unwrap();
        assert!(
            !raw.lines().any(|l| l.trim_start().starts_with("avatars")),
            "{raw}"
        );
    }

    #[test]
    fn graph_color_theme_defaults_to_follow_and_keeps_the_old_names() {
        assert_eq!(UiConfig::default().graph_color_theme, "theme");
        for t in ["gitpoulpi", "solarized", "material", "serie", "nord"] {
            let ui: UiConfig = toml::from_str(&format!("graph_color_theme = \"{t}\"")).unwrap();
            assert_eq!(ui.graph_color_slug(), t);
        }
    }

    #[test]
    fn the_graph_renderer_defaults_to_auto_and_parses_all_three_values() {
        assert_eq!(UiConfig::default().graph_renderer, GraphRenderer::Auto);
        let empty: UiConfig = toml::from_str("").unwrap();
        assert_eq!(empty.graph_renderer, GraphRenderer::Auto);
        for value in GraphRenderer::ALL {
            let ui: UiConfig = toml::from_str(&format!(
                "graph_renderer = \"{}\"
",
                value.as_str()
            ))
            .unwrap();
            assert_eq!(ui.graph_renderer, *value);
        }
    }

    #[test]
    fn hide_finished_pr_chip_defaults_on_and_round_trips() {
        assert!(UiConfig::default().hide_finished_pr_chip);
        let empty: UiConfig = toml::from_str("").unwrap();
        assert!(empty.hide_finished_pr_chip);
        let off: UiConfig = toml::from_str("hide_finished_pr_chip = false\n").unwrap();
        assert!(!off.hide_finished_pr_chip);
        let text = toml::to_string(&off).unwrap();
        assert!(text.contains("hide_finished_pr_chip = false"), "{text}");
    }

    #[test]
    fn pr_terminology_defaults_to_auto_and_round_trips() {
        assert_eq!(UiConfig::default().pr_terminology, PrTerminology::Auto);
        let empty: UiConfig = toml::from_str("").unwrap();
        assert_eq!(empty.pr_terminology, PrTerminology::Auto);
        for value in PrTerminology::ALL {
            let ui: UiConfig =
                toml::from_str(&format!("pr_terminology = \"{}\"\n", value.as_str())).unwrap();
            assert_eq!(ui.pr_terminology, *value);
            assert_eq!(PrTerminology::from_label(value.label()), Some(*value));
        }
        assert!(toml::from_str::<UiConfig>("pr_terminology = \"mr\"\n").is_err());
    }

    #[test]
    fn the_language_defaults_to_auto_with_git_terms_off_and_round_trips() {
        let ui = UiConfig::default();
        assert_eq!(ui.language, UiLanguage::Auto);
        assert!(!ui.translate_git_terms);
        assert!(!ui.git_terms_english);
        let old: UiConfig = toml::from_str("mouse_enabled = true\n").unwrap();
        assert_eq!(old.language, UiLanguage::Auto);
        for value in UiLanguage::ALL {
            let ui: UiConfig =
                toml::from_str(&format!("language = \"{}\"\n", value.as_str())).unwrap();
            assert_eq!(ui.language, *value);
        }
        assert_eq!(UiLanguage::PtBr.as_str(), "pt-BR");
        assert!(toml::from_str::<UiConfig>("language = \"klingon\"\n").is_err());
        let ui: UiConfig = toml::from_str("translate_git_terms = true\n").unwrap();
        assert!(ui.translate_git_terms);
        let ui: UiConfig = toml::from_str(
            "git_terms_english = true
",
        )
        .unwrap();
        assert!(ui.git_terms_english);
    }

    #[test]
    fn the_kitty_keyboard_defaults_to_auto_and_parses_both_values() {
        assert_eq!(UiConfig::default().kitty_keyboard, KittyKeyboard::Auto);
        let empty: UiConfig = toml::from_str("").unwrap();
        assert_eq!(empty.kitty_keyboard, KittyKeyboard::Auto);
        for value in KittyKeyboard::ALL {
            let ui: UiConfig =
                toml::from_str(&format!("kitty_keyboard = \"{}\"\n", value.as_str())).unwrap();
            assert_eq!(ui.kitty_keyboard, *value);
        }
        assert!(toml::from_str::<UiConfig>("kitty_keyboard = \"on\"\n").is_err());
    }

    #[test]
    fn the_image_protocol_defaults_to_auto_and_parses_all_five_values() {
        assert_eq!(UiConfig::default().image_protocol, ImageProtocol::Auto);
        let empty: UiConfig = toml::from_str("").unwrap();
        assert_eq!(empty.image_protocol, ImageProtocol::Auto);
        for value in ImageProtocol::ALL {
            let ui: UiConfig = toml::from_str(&format!(
                "image_protocol = \"{}\"
",
                value.as_str()
            ))
            .unwrap();
            assert_eq!(ui.image_protocol, *value);
        }
        assert!(toml::from_str::<UiConfig>(
            "image_protocol = \"regis\"
"
        )
        .is_err());
    }

    /// Same shape as the graph-renderer test above: the Diff view's layout
    /// defaults to Inline, hunks-only to on, and every layout spelling
    /// round-trips through TOML.
    #[test]
    fn the_diff_layout_defaults_to_inline_hunks_only_and_parses_both_values() {
        assert_eq!(UiConfig::default().diff_layout, DiffLayout::Inline);
        assert!(UiConfig::default().diff_hunks_only);
        let empty: UiConfig = toml::from_str("").unwrap();
        assert_eq!(empty.diff_layout, DiffLayout::Inline);
        assert!(empty.diff_hunks_only);
        for value in DiffLayout::ALL {
            let ui: UiConfig =
                toml::from_str(&format!("diff_layout = \"{}\"\n", value.as_str())).unwrap();
            assert_eq!(ui.diff_layout, *value);
        }
        assert!(toml::from_str::<UiConfig>("diff_layout = \"hunk\"\n").is_err());
    }

    /// The old three-valued `ui.diff_view_mode` migrates so the user sees
    /// what they saw - all three showed only the hunks, `split` in two
    /// columns. Red without the `migrate_legacy` call in
    /// `crate::Config::from_toml_str`.
    #[test]
    fn the_legacy_diff_view_mode_migrates_to_layout_and_hunks_only() {
        let load = |body: &str| crate::Config::from_toml_str(body).unwrap().0.ui;
        for (old, layout) in [
            ("hunk", DiffLayout::Inline),
            ("inline", DiffLayout::Inline),
            ("split", DiffLayout::Split),
        ] {
            let ui = load(&format!("[ui]\ndiff_view_mode = \"{old}\"\n"));
            assert_eq!(ui.diff_layout, layout, "{old}");
            assert!(ui.diff_hunks_only, "{old} showed only the hunks");
        }
        // A new key in the file wins over the old one still beside it.
        let ui = load(
            "[ui]\ndiff_view_mode = \"split\"\ndiff_layout = \"inline\"\ndiff_hunks_only = false\n",
        );
        assert_eq!(ui.diff_layout, DiffLayout::Inline);
        assert!(!ui.diff_hunks_only);
        // And the old key is never written back.
        let (config, _) =
            crate::Config::from_toml_str("[ui]\ndiff_view_mode = \"split\"\n").unwrap();
        let saved = toml::to_string_pretty(&config).unwrap();
        assert!(!saved.contains("diff_view_mode"), "{saved}");
        assert!(saved.contains("diff_layout = \"split\""), "{saved}");
    }

    /// The migration also runs on the path the TUI resolves config
    /// through (`Settings::resolve_from`), not only `Config::load`.
    #[test]
    fn settings_resolution_sees_the_migrated_layout() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("config.toml");
        std::fs::write(&path, "[ui]\ndiff_view_mode = \"split\"\n").unwrap();
        let settings =
            crate::Settings::resolve_from(Some(&path), |_| None, &crate::Overrides::default())
                .unwrap();
        assert_eq!(settings.config().ui.diff_layout, DiffLayout::Split);
        assert_eq!(
            crate::Config::load(&path).unwrap().ui.diff_layout,
            DiffLayout::Split
        );
    }

    /// The Pull button defaults to the usual choice, Push goes
    /// straight to the remote, and every spelling `as_str` produces is
    /// one `set_in_file` accepts back - the dropdown writes exactly that.
    #[test]
    fn pull_mode_and_push_behavior_defaults_and_spellings_round_trip() {
        let empty: crate::Config = toml::from_str("").unwrap();
        assert_eq!(empty.git.pull_mode, PullButtonMode::Merge);
        assert_eq!(empty.git.push_behavior, PushBehavior::Direct);
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("config.toml");
        for mode in PullButtonMode::ALL {
            crate::set_in_file(&path, "git.pull_mode", mode.as_str()).unwrap();
            assert_eq!(crate::Config::load(&path).unwrap().git.pull_mode, *mode);
        }
        for behavior in PushBehavior::ALL {
            crate::set_in_file(&path, "git.push_behavior", behavior.as_str()).unwrap();
            assert_eq!(
                crate::Config::load(&path).unwrap().git.push_behavior,
                *behavior
            );
        }
    }

    #[test]
    fn diff_show_whitespace_defaults_true_and_diff_word_wrap_defaults_false() {
        // A config-free install gets the Diff view's own defaults.
        let empty: UiConfig = toml::from_str("").unwrap();
        assert!(empty.diff_show_whitespace);
        assert!(!empty.diff_word_wrap);
        assert!(!empty.minimap, "the minimap is opt-in");
    }

    /// Three rows per wheel notch, like every other scrollable surface in
    /// the TUI; and a `0` in a config file must not quietly turn the wheel
    /// off.
    #[test]
    fn the_wheel_step_defaults_to_three_and_is_clamped() {
        let mut ui = UiConfig::default();
        assert_eq!(ui.mouse_scroll_lines_clamped(), 3);
        ui.mouse_scroll_lines = 0;
        assert_eq!(ui.mouse_scroll_lines_clamped(), MIN_MOUSE_SCROLL_LINES);
        ui.mouse_scroll_lines = 500;
        assert_eq!(ui.mouse_scroll_lines_clamped(), MAX_MOUSE_SCROLL_LINES);
        let empty: UiConfig = toml::from_str("").unwrap();
        assert_eq!(empty.mouse_scroll_lines_clamped(), 3);
    }

    #[test]
    fn an_out_of_range_graph_limit_is_clamped_not_obeyed() {
        let mut general = GeneralConfig {
            initial_commits_in_graph: 0,
            ..GeneralConfig::default()
        };
        assert_eq!(
            general.initial_commits_in_graph_clamped(),
            MIN_INITIAL_COMMITS_IN_GRAPH
        );
        general.initial_commits_in_graph = 12_000;
        assert_eq!(general.initial_commits_in_graph_clamped(), 12_000);
    }

    /// The page size has a floor for the same reason the initial load
    /// does, and the ceiling has two states that are easy to get wrong:
    /// `0` means *no ceiling* (not *load nothing*), and a ceiling below
    /// the initial load would make the very first request illegal.
    #[test]
    fn the_lazy_page_size_is_floored_and_the_ceiling_never_undercuts_the_first_page() {
        let mut general = GeneralConfig {
            commits_per_lazy_page: 0,
            ..GeneralConfig::default()
        };
        assert_eq!(
            general.commits_per_lazy_page_clamped(),
            MIN_COMMITS_PER_LAZY_PAGE
        );
        general.commits_per_lazy_page = 2_500;
        assert_eq!(general.commits_per_lazy_page_clamped(), 2_500);

        assert_eq!(
            GeneralConfig::default().max_commits_in_graph_effective(),
            Some(DEFAULT_MAX_COMMITS_IN_GRAPH)
        );
        general.max_commits_in_graph = 0;
        assert_eq!(
            general.max_commits_in_graph_effective(),
            None,
            "zero is `no ceiling`, not `hold nothing`"
        );
        general.max_commits_in_graph = 10;
        assert_eq!(
            general.max_commits_in_graph_effective(),
            Some(general.initial_commits_in_graph_clamped()),
            "a ceiling under the initial load would make the first request illegal"
        );
    }

    #[test]
    fn the_radar_is_enabled_by_default_and_round_trips_off() {
        let general = GeneralConfig::default();
        assert!(general.radar_enabled);
        let (config, _) =
            crate::Config::from_toml_str("[general]\nradar_enabled = false\n").unwrap();
        assert!(!config.general.radar_enabled);
        let (absent, _) =
            crate::Config::from_toml_str("[general]\nremember_tabs = true\n").unwrap();
        assert!(absent.general.radar_enabled, "absent means on");
        let text = toml::to_string(&config).unwrap();
        let (back, _) = crate::Config::from_toml_str(&text).unwrap();
        assert!(!back.general.radar_enabled);
    }

    #[test]
    fn the_startup_tabs_default_to_todays_behaviour_and_round_trip() {
        let general = GeneralConfig::default();
        assert_eq!(general.startup_tab_in_repo, StartupTab::CurrentRepo);
        assert_eq!(general.startup_tab_outside_repo, StartupTab::Home);
        assert_eq!(general.startup_tab_launcher, StartupTab::Last);
        let (config, _) = crate::Config::from_toml_str(
            "[general]\nstartup_tab_in_repo = \"radar\"\n\
             startup_tab_outside_repo = \"last\"\nstartup_tab_launcher = \"home\"\n",
        )
        .unwrap();
        assert_eq!(config.general.startup_tab_in_repo, StartupTab::Radar);
        assert_eq!(config.general.startup_tab_outside_repo, StartupTab::Last);
        let (absent, _) =
            crate::Config::from_toml_str("[general]\nremember_tabs = true\n").unwrap();
        assert_eq!(absent.general.startup_tab_in_repo, StartupTab::CurrentRepo);
        let text = toml::to_string(&config).unwrap();
        let (back, _) = crate::Config::from_toml_str(&text).unwrap();
        assert_eq!(back.general.startup_tab_in_repo, StartupTab::Radar);
        for tab in StartupTab::ALL {
            let (c, _) = crate::Config::from_toml_str(&format!(
                "[general]\nstartup_tab_launcher = \"{}\"\n",
                tab.as_str()
            ))
            .unwrap();
            assert_eq!(c.general.startup_tab_launcher, *tab);
        }
        assert!(
            crate::Config::from_toml_str("[general]\nstartup_tab_launcher = \"tab3\"\n").is_err()
        );
    }

    #[test]
    fn auto_fetch_is_off_at_zero_and_capped_at_an_hour() {
        let mut general = GeneralConfig::default();
        // Default: every 5 minutes, and a fetch when a tab opens.
        assert_eq!(general.auto_fetch_interval_minutes, 5);
        assert!(general.fetch_on_tab_open);
        assert_eq!(
            general.auto_fetch_interval(),
            Some(std::time::Duration::from_secs(300))
        );
        general.auto_fetch_interval_minutes = 0;
        assert_eq!(general.auto_fetch_interval(), None);
        general.auto_fetch_interval_minutes = 5;
        general.auto_fetch_interval_minutes = 9_999;
        assert_eq!(
            general.auto_fetch_interval(),
            Some(std::time::Duration::from_secs(3600))
        );
    }

    /// The folder picker's "Show sibling folders" is off by default and
    /// survives a save and a reload (a temp config, never the real one).
    #[test]
    fn show_siblings_defaults_off_and_persists() {
        let empty: crate::Config = toml::from_str("").unwrap();
        assert!(!empty.repos.show_siblings);
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("config.toml");
        crate::set_in_file(&path, "repos.show_siblings", "true").unwrap();
        assert!(crate::Config::load(&path).unwrap().repos.show_siblings);
        crate::set_in_file(&path, "repos.show_siblings", "false").unwrap();
        assert!(!crate::Config::load(&path).unwrap().repos.show_siblings);
    }

    #[test]
    fn editor_defaults_round_trip_and_clamp() {
        let empty: crate::Config = toml::from_str("").unwrap();
        assert_eq!(empty.editor, EditorConfig::default());
        assert_eq!(empty.editor.tab_size, 4);
        assert_eq!(empty.editor.eol, EditorEol::Auto);
        assert!(empty.editor.syntax_highlighting && empty.editor.line_numbers);
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("config.toml");
        crate::set_in_file(&path, "editor.tab_size", "2").unwrap();
        crate::set_in_file(&path, "editor.eol", "crlf").unwrap();
        crate::set_in_file(&path, "editor.syntax_highlighting", "false").unwrap();
        crate::set_in_file(&path, "editor.line_numbers", "false").unwrap();
        assert!(crate::set_in_file(&path, "editor.eol", "cr").is_err());
        let loaded = crate::Config::load(&path).unwrap().editor;
        assert_eq!(loaded.tab_size, 2);
        assert_eq!(loaded.eol, EditorEol::Crlf);
        assert!(!loaded.syntax_highlighting && !loaded.line_numbers);
        let wild: EditorConfig = toml::from_str("tab_size = 0\n").unwrap();
        assert_eq!(wild.tab_size_clamped(), 1);
        let wild: EditorConfig = toml::from_str("tab_size = 99\n").unwrap();
        assert_eq!(wild.tab_size_clamped(), 8);
        for e in EditorEol::ALL {
            assert_eq!(EditorEol::from_label(e.label()), Some(*e));
        }
    }

    #[test]
    fn the_row_tint_strength_parses_labels_and_defaults_to_medium() {
        #[derive(Deserialize)]
        struct W {
            v: GraphRowTintStrength,
        }
        for (name, want) in [
            ("strong", GraphRowTintStrength::Strong),
            ("medium", GraphRowTintStrength::Medium),
            ("subtle", GraphRowTintStrength::Subtle),
        ] {
            let w: W = toml::from_str(&format!("v = \"{name}\"")).unwrap();
            assert_eq!(w.v, want);
            assert_eq!(want.as_str(), name);
            assert_eq!(GraphRowTintStrength::from_label(want.label()), Some(want));
        }
        assert!(toml::from_str::<W>("v = \"loud\"").is_err());
        assert_eq!(
            GraphRowTintStrength::default(),
            GraphRowTintStrength::Medium
        );
        assert_eq!(
            UiConfig::default().graph_row_tint_strength,
            GraphRowTintStrength::Medium
        );
        let ui: UiConfig = toml::from_str("graph_row_tint_strength = \"subtle\"").unwrap();
        assert_eq!(ui.graph_row_tint_strength, GraphRowTintStrength::Subtle);
    }

    #[test]
    fn lane_background_labels_are_friendly_and_keep_the_stored_names() {
        assert_eq!(GraphLaneBackground::Off.label(), "Off");
        assert_eq!(GraphLaneBackground::RowTint.label(), "Row tint");
        assert_eq!(GraphLaneBackground::RowTint.as_str(), "row_tint");
        for b in GraphLaneBackground::ALL {
            assert_eq!(GraphLaneBackground::from_label(b.label()), Some(*b));
        }
    }
}
