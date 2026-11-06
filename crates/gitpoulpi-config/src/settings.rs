//! The precedence ladder that turns the three places a setting can come
//! from into the one value a component acts on.
//!
//! **Lowest to highest: built-in defaults -> `config.toml` -> `GITPOULPI_*`
//! environment variables -> command-line flags.** A higher layer that says
//! nothing leaves the layer below it in force; a higher layer that says
//! something *wrong* is an error, never a silent fall-through, so a user
//! can always tell a misspelled key from a feature that was never built.
//!
//! `GITPOULPI_CONFIG_DIR` is not part of this ladder: it selects which
//! `config.toml` the file layer reads, so it sits underneath the whole
//! stack (see `paths.rs`).
//!
//! **Only the keys in [`LIVE_KEYS`] are actually consulted by any
//! component.** Everything else in [`crate::Config`] parses, round-trips
//! and can be read back with `gitpoulpi config get` (which reports that per
//! key; `docs/user/config-reference.md` marks them), but nothing reads it.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use crate::{Config, ConfigPaths, DefaultBackend, Error, Result};

/// Prefix for every environment-variable override.
pub const ENV_PREFIX: &str = "GITPOULPI_";

/// The dotted keys some component genuinely reads today.
///
/// Kept as data rather than as scattered knowledge so `gitpoulpi config
/// get` can tell a user the truth about a key without anyone having to
/// remember to update a doc.
pub const LIVE_KEYS: &[&str] = &[
    "general.default_backend",
    "general.initial_commits_in_graph",
    "general.lazy_load_commits_in_graph",
    "general.commits_per_lazy_page",
    "general.max_commits_in_graph",
    "general.auto_fetch_interval_minutes",
    "general.auto_fetch_interval_unit",
    "general.fetch_on_tab_open",
    "general.hosting_refresh_minutes",
    "general.hosting_refresh_unit",
    "general.default_branch_name",
    "general.auto_prune",
    "general.auto_prune_tags",
    "general.delete_orig_files_after_merge",
    "general.remember_tabs",
    "general.radar_enabled",
    "general.startup_tab_in_repo",
    "general.startup_tab_outside_repo",
    "general.startup_tab_launcher",
    "general.group_worktree_tabs",
    "general.active_profile",
    "general.external_editor",
    "general.external_diff_tool",
    "general.external_merge_tool",
    "general.file_explorer",
    "general.undo_depth",
    "general.undo_enabled",
    "general.journal_outside",
    "general.conflict_detection",
    "general.dev_mode",
    "general.auto_stash",
    "general.keep_submodules_up_to_date",
    "general.include_untracked_in_autostash",
    "log.level",
    "log.retention_days",
    "log.retention_unit",
    "log.max_file_mb",
    "log.max_file_unit",
    "log.timings",
    "log.extended",
    "log.commit_count",
    "log.use_mailmap",
    "search.commit_limit",
    "rewrite.protected_branches",
    "rewrite.external_scripts",
    "rewrite.script_timeout_ms",
    "rewrite.script_timeout_unit",
    "rewrite.script_total_timeout_s",
    "rewrite.script_total_timeout_unit",
    "updates.mode",
    "updates.channel",
    "ui.language",
    "ui.theme",
    "ui.translate_git_terms",
    "ui.git_terms_english",
    "ui.mouse_enabled",
    "ui.date_format",
    "ui.date_locale",
    "ui.date_time_format",
    "ui.date_verbose_format",
    "ui.graph_lane_background",
    "ui.graph_row_tint_strength",
    "ui.toolbar_position",
    "ui.toolbar_align",
    "ui.toolbar_pr_pinned",
    "ui.statusbar_position",
    "ui.tabs_position",
    "ui.graph_color_theme",
    "ui.ref_tree",
    "ui.graph_renderer",
    "ui.image_protocol",
    "ui.kitty_keyboard",
    "ui.color_depth",
    "ui.graph_show_stashes",
    "ui.ref_connector",
    "ui.log_wip_counts_in_message",
    "ui.reveal_on_ref_click",
    "ui.graph_ghost_ref",
    "ui.allow_agent_view_control",
    "ui.launch",
    "ui.launch_command",
    "ui.tab_dir_follow",
    "ui.terminal_launch",
    "ui.terminal_launch_command",
    "ui.open_in",
    "ui.open_in_default",
    "ui.launch_terminal",
    "ui.launcher_build",
    "ui.relaunch_in_terminal",
    "ui.reuse_window",
    "ui.command_alias",
    "ui.command_alias_add",
    "ui.image_view",
    "ui.image_view_max_mib",
    "ui.image_view_max_unit",
    "ui.image_view_text_fallback",
    "ui.image_view_pixelated_from",
    "ui.image_view_sync",
    "ui.image_editors",
    "ui.image_editor_default",
    "ui.logo_color",
    "ui.mouse_scroll_lines",
    "ui.show_avatars",
    "ui.avatars",
    "ui.avatar_sources",
    "ui.avatar_fallback",
    "ui.composer_files",
    "ui.sidebar_sections_scope",
    "ui.blame_side",
    "ui.column_resize",
    "ui.log_columns_layout",
    "ui.blame_width",
    "ui.diff_layout",
    "ui.pr_terminology",
    "ui.hide_finished_pr_chip",
    "ui.diff_hunks_only",
    "ui.diff_show_whitespace",
    "ui.diff_word_wrap",
    "ui.minimap",
    "ui.mermaid_detail",
    "ui.mermaid_orientation",
    "ui.mermaid_fence",
    "ui.mermaid_subject_width",
    "ui.input_color",
    "ui.input_color_custom",
    "editor.tab_size",
    "editor.eol",
    "editor.syntax_highlighting",
    "editor.line_numbers",
    "editor.follow_selection",
    "merge.side_by_side_min_width",
    "git.pull_mode",
    "git.default_encoding",
    "credentials.https_mode",
    "git.lfs_enabled",
    "git.lfs_autofix_filters",
    "git.push_behavior",
    "ssh.key_type",
    "ssh.private_key_path",
    "ssh.public_key_path",
    "repos.clone_root",
    "repos.clone_start_source",
    "repos.browse_root",
    "session.per_repo_state_dir",
    "repos.show_hidden",
    "repos.show_siblings",
    "mcp.enabled",
    "mcp.bind",
    "mcp.allow_mutations",
    "mcp.mutation_allowlist",
    "mcp.allow_ui",
    "mcp.ui_control",
    "mcp.allow_ui_propose",
    "mcp.http_auth",
    "mcp.tls",
    "mcp.tls_cert",
    "mcp.tls_key",
    "mcp.behind_proxy",
    "mcp.allow_sensitive_staging",
    "ai.enabled",
    "ai.default_profile",
    "ai.global_instructions",
    "ai.review.run_in_repository",
    "ai.review.include_claude_md",
    "ai.review.include_agents_md",
    "ai.review.extra_file",
    "ai.privacy.redact_secrets",
    "ai.privacy.exclude_sensitive_files",
    "ai.sent_preview.commit_message",
    "ai.sent_preview.explain_changes",
    "ai.sent_preview.stash_message",
    "ai.sent_preview.pull_request_description",
    "ai.sent_preview.commit_composer",
    "ai.sent_preview.conflict_resolve",
    "ai.sent_preview.code_review",
    "ai.sent_preview.split_commit",
    "ai.sent_preview.rewrite_script",
    "ai.sent_preview.rewrite_rules",
    "privacy.sensitive_patterns",
    "privacy.staging_guard",
    "ai.actions.commit_message.profile",
    "ai.actions.commit_message.instructions",
    "ai.actions.explain_changes.profile",
    "ai.actions.explain_changes.instructions",
    "ai.actions.stash_message.profile",
    "ai.actions.stash_message.instructions",
    "ai.actions.pull_request_description.profile",
    "ai.actions.pull_request_description.instructions",
    "ai.actions.commit_composer.profile",
    "ai.actions.commit_composer.instructions",
    "ai.actions.conflict_resolve.profile",
    "ai.actions.conflict_resolve.instructions",
];

/// The environment variable that overrides a dotted config key.
///
/// Mechanical, so it never has to be looked up: uppercase the key and
/// turn the dots into underscores. `general.default_backend` is
/// `GITPOULPI_GENERAL_DEFAULT_BACKEND`.
pub fn env_var_for(key: &str) -> String {
    format!("{ENV_PREFIX}{}", key.replace('.', "_").to_uppercase())
}

/// Which layer supplied the value currently in force for a key.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Source {
    Default,
    File,
    Env,
    Flag,
}

impl Source {
    pub fn as_str(self) -> &'static str {
        match self {
            Source::Default => "default",
            Source::File => "file",
            Source::Env => "env",
            Source::Flag => "flag",
        }
    }
}

/// The command-line layer: one field per flag that overrides a config key.
///
/// `None` means "the flag was not passed", which is not the same as the
/// flag's own default - that distinction is what keeps a flag from
/// silently outranking a `config.toml` the user never mentioned.
#[derive(Debug, Clone, Default)]
pub struct Overrides {
    /// `--backend git2|system`.
    pub default_backend: Option<DefaultBackend>,
}

/// A fully resolved configuration, plus where each value came from.
#[derive(Debug)]
pub struct Settings {
    config: Config,
    file: Option<PathBuf>,
    /// The file layer as raw TOML, kept so `source_of` can distinguish
    /// "the file sets this key to the default" from "the file is silent".
    /// Comparing values instead would misreport the former.
    raw_file: Option<toml::Value>,
    overridden: BTreeMap<String, Source>,
}

impl Settings {
    /// The whole ladder, against the real process environment.
    ///
    /// `paths` is `None` only when the OS gave us no config directory at
    /// all, in which case there is no file layer to read and the defaults
    /// plus env plus flags still apply.
    pub fn resolve(paths: Option<&ConfigPaths>, overrides: &Overrides) -> Result<Self> {
        let file = paths.map(|p| p.config_file());
        Self::resolve_from(file.as_deref(), |name| std::env::var(name).ok(), overrides)
    }

    /// The ladder with no command-line layer: what a component that has no
    /// flags of its own (the TUI) resolves.
    pub fn resolve_ambient() -> Result<Self> {
        Self::resolve(ConfigPaths::discover().as_ref(), &Overrides::default())
    }

    /// The testable core. `env` is a lookup rather than `std::env` because
    /// process environment is global mutable state: a test that set a real
    /// variable would be visible to every other test running in the same
    /// process at the same time.
    pub fn resolve_from(
        file: Option<&Path>,
        env: impl Fn(&str) -> Option<String>,
        overrides: &Overrides,
    ) -> Result<Self> {
        let (config, raw_file) = match file {
            Some(path) if path.exists() => {
                let raw = std::fs::read_to_string(path)?;
                let (config, doc) = Config::from_toml_str(&raw)?;
                (config, Some(doc))
            }
            _ => (Config::default(), None),
        };

        let mut settings = Self {
            config,
            file: file.map(Path::to_path_buf),
            raw_file,
            overridden: BTreeMap::new(),
        };

        // Env before flags, so a flag wins a tie.
        for key in LIVE_KEYS {
            if let Some(value) = env(&env_var_for(key)) {
                settings.config.set(key, &value).map_err(|err| match err {
                    // Reported against the variable the user actually
                    // typed, not against the dotted key they never saw.
                    Error::BadValue {
                        value, expected, ..
                    } => Error::BadValue {
                        key: env_var_for(key),
                        value,
                        expected,
                    },
                    other => other,
                })?;
                settings.overridden.insert((*key).to_string(), Source::Env);
            }
        }

        if let Some(backend) = overrides.default_backend {
            settings.config.general.default_backend = backend;
            settings
                .overridden
                .insert("general.default_backend".to_string(), Source::Flag);
        }

        Ok(settings)
    }

    pub fn config(&self) -> &Config {
        &self.config
    }

    /// The `config.toml` this resolved against, whether or not it exists.
    pub fn file(&self) -> Option<&Path> {
        self.file.as_deref()
    }

    /// Which layer the value in force for `key` came from.
    pub fn source_of(&self, key: &str) -> Source {
        if let Some(source) = self.overridden.get(key) {
            return *source;
        }
        if self.key_is_in_file(key) {
            return Source::File;
        }
        Source::Default
    }

    /// Whether some component actually reads `key` today.
    pub fn key_is_implemented(key: &str) -> bool {
        LIVE_KEYS.contains(&key) || key.starts_with("keybindings.")
    }

    fn key_is_in_file(&self, key: &str) -> bool {
        let mut cursor = match &self.raw_file {
            Some(doc) => doc,
            None => return false,
        };
        for segment in key.split('.') {
            match cursor.get(segment) {
                Some(next) => cursor = next,
                None => return false,
            }
        }
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn no_env(_: &str) -> Option<String> {
        None
    }

    fn write(dir: &tempfile::TempDir, body: &str) -> PathBuf {
        let path = dir.path().join("config.toml");
        std::fs::write(&path, body).unwrap();
        path
    }

    #[test]
    fn the_env_variable_for_a_key_is_mechanical() {
        assert_eq!(
            env_var_for("general.default_backend"),
            "GITPOULPI_GENERAL_DEFAULT_BACKEND"
        );
        assert_eq!(
            env_var_for("ui.mouse_enabled"),
            "GITPOULPI_UI_MOUSE_ENABLED"
        );
    }

    #[test]
    fn with_no_file_every_value_is_the_built_in_default() {
        let settings = Settings::resolve_from(None, no_env, &Overrides::default()).unwrap();
        assert_eq!(
            settings.config().general.default_backend,
            DefaultBackend::Git2
        );
        assert_eq!(
            settings.source_of("general.default_backend"),
            Source::Default
        );
    }

    #[test]
    fn a_missing_file_is_not_an_error_just_an_absent_layer() {
        let dir = tempfile::tempdir().unwrap();
        let settings = Settings::resolve_from(
            Some(&dir.path().join("config.toml")),
            no_env,
            &Overrides::default(),
        )
        .unwrap();
        assert_eq!(
            settings.source_of("general.default_backend"),
            Source::Default
        );
    }

    #[test]
    fn the_file_layer_beats_the_defaults() {
        let dir = tempfile::tempdir().unwrap();
        let path = write(&dir, "[general]\ndefault_backend = \"system\"\n");
        let settings = Settings::resolve_from(Some(&path), no_env, &Overrides::default()).unwrap();
        assert_eq!(
            settings.config().general.default_backend,
            DefaultBackend::System
        );
        assert_eq!(settings.source_of("general.default_backend"), Source::File);
    }

    /// The reason `source_of` reads the raw TOML instead of comparing the
    /// resolved value against `Config::default()`: a file that spells out
    /// the default value is still the file speaking.
    #[test]
    fn a_file_that_repeats_the_default_still_reads_as_the_file_layer() {
        let dir = tempfile::tempdir().unwrap();
        let path = write(&dir, "[general]\ndefault_backend = \"git2\"\n");
        let settings = Settings::resolve_from(Some(&path), no_env, &Overrides::default()).unwrap();
        assert_eq!(settings.source_of("general.default_backend"), Source::File);
    }

    #[test]
    fn the_env_layer_beats_the_file() {
        let dir = tempfile::tempdir().unwrap();
        let path = write(&dir, "[general]\ndefault_backend = \"git2\"\n");
        let settings = Settings::resolve_from(
            Some(&path),
            |name| (name == "GITPOULPI_GENERAL_DEFAULT_BACKEND").then(|| "system".to_string()),
            &Overrides::default(),
        )
        .unwrap();
        assert_eq!(
            settings.config().general.default_backend,
            DefaultBackend::System
        );
        assert_eq!(settings.source_of("general.default_backend"), Source::Env);
    }

    #[test]
    fn the_flag_layer_beats_the_env_layer() {
        let dir = tempfile::tempdir().unwrap();
        let path = write(&dir, "[general]\ndefault_backend = \"git2\"\n");
        let settings = Settings::resolve_from(
            Some(&path),
            |name| (name == "GITPOULPI_GENERAL_DEFAULT_BACKEND").then(|| "git2".to_string()),
            &Overrides {
                default_backend: Some(DefaultBackend::System),
            },
        )
        .unwrap();
        assert_eq!(
            settings.config().general.default_backend,
            DefaultBackend::System
        );
        assert_eq!(settings.source_of("general.default_backend"), Source::Flag);
    }

    #[test]
    fn a_boolean_env_override_applies_to_ui_keys_too() {
        let settings = Settings::resolve_from(
            None,
            |name| (name == "GITPOULPI_UI_MOUSE_ENABLED").then(|| "false".to_string()),
            &Overrides::default(),
        )
        .unwrap();
        assert!(!settings.config().ui.mouse_enabled);
        assert_eq!(settings.source_of("ui.mouse_enabled"), Source::Env);
    }

    /// A misspelled env value must not quietly leave the file's value in
    /// place - that is indistinguishable from the variable being ignored,
    /// which is the class of bug this module exists to remove.
    #[test]
    fn a_bad_env_value_is_an_error_naming_the_variable_the_user_typed() {
        let err = Settings::resolve_from(
            None,
            |name| (name == "GITPOULPI_GENERAL_DEFAULT_BACKEND").then(|| "mercurial".to_string()),
            &Overrides::default(),
        )
        .unwrap_err();
        let message = err.to_string();
        assert!(
            message.contains("GITPOULPI_GENERAL_DEFAULT_BACKEND"),
            "{message}"
        );
        assert!(message.contains("mercurial"), "{message}");
    }

    #[test]
    fn an_unparseable_config_file_is_an_error_not_a_silent_default() {
        let dir = tempfile::tempdir().unwrap();
        let path = write(&dir, "[general\ndefault_backend = \"system\"\n");
        assert!(Settings::resolve_from(Some(&path), no_env, &Overrides::default()).is_err());
    }

    #[test]
    fn only_the_keys_something_reads_are_advertised_as_implemented() {
        assert!(Settings::key_is_implemented("general.default_backend"));
        assert!(Settings::key_is_implemented("ui.mouse_enabled"));
        assert!(Settings::key_is_implemented("general.auto_stash"));
        assert!(Settings::key_is_implemented("ui.theme"));
        assert!(Settings::key_is_implemented("mcp.enabled"));
        assert!(Settings::key_is_implemented("mcp.bind"));
        assert!(Settings::key_is_implemented("mcp.allow_mutations"));
        assert!(Settings::key_is_implemented("mcp.mutation_allowlist"));
    }

    /// The Diff view's three toggles have to be advertised as implemented -
    /// the Preferences screen derives its `not wired yet` note straight
    /// from `LIVE_KEYS`, so a key missing here would tell the user this
    /// setting does nothing while it actually restores the last-used view
    /// on every diff opened.
    /// Git LFS support is real: `git.lfs_enabled` gates the TUI's LFS
    /// display and every LFS step GitPoulpi runs itself, and it is on by
    /// default.
    #[test]
    fn git_lfs_enabled_is_implemented_and_on_by_default() {
        assert!(Settings::key_is_implemented("git.lfs_enabled"));
        assert!(Config::default().git.lfs_enabled);
        assert!(Settings::key_is_implemented("git.lfs_autofix_filters"));
        assert!(Config::default().git.lfs_autofix_filters);
    }

    #[test]
    fn the_diff_view_toggles_are_advertised_as_implemented() {
        assert!(Settings::key_is_implemented("ui.diff_layout"));
        assert!(Settings::key_is_implemented("ui.diff_hunks_only"));
        assert!(Settings::key_is_implemented("ui.diff_show_whitespace"));
        assert!(Settings::key_is_implemented("ui.diff_word_wrap"));
    }
}
