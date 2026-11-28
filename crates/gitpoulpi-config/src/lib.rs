mod ai;
pub mod browser;
pub mod desktop;
pub mod dev_mode;
pub mod image_editors;
pub mod installation;
mod integration;
mod keybindings;
pub mod launch;
pub mod mcp_port;
pub mod mcp_secrets;
pub mod open_in;
pub mod os_apps;
pub mod osc52;
mod paths;
pub mod policy;
pub mod release;
mod repo_data;
mod schema;
pub mod secure_fs;
pub mod sequences;
mod session_name;
mod settings;
pub mod split;
pub mod theme;
pub mod update_lock;

pub use ai::{
    AgentRunMode, AgentWorkdir, AiAction, AiActionSettings, AiActions, AiConfig, AiHostingReply,
    AiLimits, AiPreset, AiPrivacy, AiProfile, AiProviderKind, AiResolveError, AiResolved,
    AiReviewContext, AiSentPreview, RepoAiHostingReply, RepoAiInstructions, RepoAiReview,
    RepoInstructionsMode, RunInRepo, WorkspaceAi, DEFAULT_CONFLICT_BYTES,
    DEFAULT_CONTEXT_TTL_MINUTES, DEFAULT_DIFF_BYTES, DEFAULT_HEADLESS_TIMEOUT_SECS,
    DEFAULT_HOSTING_REPLY_CONTEXT_BYTES, DEFAULT_HOSTING_REPLY_FILE_BYTES,
    DEFAULT_HOSTING_REPLY_HITS_PER_TERM, DEFAULT_HOSTING_REPLY_MAP_BYTES,
    DEFAULT_HOSTING_REPLY_TERMS, DEFAULT_PROFILE_ID, DEFAULT_REVIEW_CONTEXT_BYTES,
    DEFAULT_REVIEW_CONTEXT_FILE_BYTES, DEFAULT_REVIEW_FILE_BYTES, DEFAULT_REVIEW_TEXT_BYTES,
    DEFAULT_REVIEW_THREADS_BYTES, DEFAULT_TERMINAL_TIMEOUT_SECS, MAX_CONFLICT_BYTES,
    MAX_CONTEXT_TTL_MINUTES, MAX_HOSTING_REPLY_HITS, MAX_HOSTING_REPLY_TERMS, MAX_LIMIT_BYTES,
    MIN_CONFLICT_BYTES, MIN_HOSTING_REPLY_HITS, MIN_HOSTING_REPLY_TERMS, MIN_LIMIT_BYTES,
};
pub use installation::set_installation_in_file;
pub use integration::{
    clean_namespaces, normalize_azure_url, parse_namespace_text, AccountField, ApiFamily,
    AuthMethod, Capabilities, ConnectedUser, Integration, IntegrationCategory, IntegrationKind,
    IntegrationRadar, IntegrationSsh, LocalMarkdownSettings, RedmineMarkup, RemoteReviewMode,
    SshMode, TokenHelp, TokenLink, DEFAULT_STATUSES,
};
pub use keybindings::{
    reset_all_keybindings_in_file, reset_keybinding_in_file, set_keybinding_in_file,
    KeybindingsConfig,
};
pub use open_in::OpenInEntry;
pub use paths::{ConfigPaths, CONFIG_DIR_ENV};
pub use repo_data::{
    git_common_dir, migrate_legacy, repo_data_dir, repo_data_dir_with, repo_data_file,
    repo_data_file_with, user_data_folder_name, RepoDataLocation, DEFAULT_REPO_DATA_LOCATION,
    LEGACY_DIR_NAME,
};
pub use schema::{default_clone_root, home_dir, CLONE_START_LAST, CLONE_START_URL};
pub use schema::{
    read_quantity, AutoCrlf, AutoStashMode, AvatarFallback, AvatarPlacement, AvatarSources,
    BaseList, BaseOrigin, BlameSide, ColorDepth, ColumnResize, CommitCountMethod, ComposerFiles,
    ConflictPreventionConfig, CredentialsConfig, DefaultBackend, DialogGeometry, DiffLayout,
    DisplayUnit, EditorConfig, EditorEol, EolMode, GeneralConfig, GitConfig, GraphColorization,
    GraphLaneBackground, GraphRenderer, GraphRowTintStrength, HostingInstance, HttpsMode,
    ImagePixelatedFrom, ImageProtocol, ImageTextFallback, InstallationConfig, InstallationEntry,
    Kind, KittyKeyboard, LauncherBuild, LegacyDiffViewMode, LogColumnsLayout, LogConfig, LogLevel,
    McpConfig, McpTls, MergeConfig, PrTerminology, PrivacyConfig, Profile, PullButtonMode,
    PushBehavior, ReposConfig, ResolvedBases, RetentionUnit, RewriteConfig, SearchConfig,
    SessionConfig, SidebarSectionsScope, SshConfig, StartupTab, StatusbarPosition, TabDirFollow,
    TabsPosition, TextCursor, ToolbarAlign, ToolbarPosition, UiBefore, UiConfig, UiLanguage,
    UpdateChannel, UpdateMode, UpdatesConfig, BASE_SOURCE_DEFAULT, BASE_SOURCE_LIST,
    BASE_SOURCE_OWN, DEFAULT_BLAME_WIDTH, DEFAULT_COMMITS_PER_LAZY_PAGE, DEFAULT_EDITOR_TAB_SIZE,
    DEFAULT_HOSTING_REFRESH_MINUTES, DEFAULT_INITIAL_COMMITS_IN_GRAPH, DEFAULT_LOG_RETENTION_DAYS,
    DEFAULT_MAX_COMMITS_IN_GRAPH, DEFAULT_MERGE_SIDE_BY_SIDE_MIN_WIDTH, DEFAULT_MOUSE_SCROLL_LINES,
    DEFAULT_SEARCH_COMMIT_LIMIT, DEFAULT_UPDATES_REPO, GRAPH_COLOR_FOLLOW_THEME,
    MAX_AUTO_FETCH_INTERVAL_MINUTES, MAX_BLAME_WIDTH, MAX_EDITOR_TAB_SIZE,
    MAX_HOSTING_REFRESH_MINUTES, MAX_IMAGE_VIEW_MAX_MIB, MAX_LOG_MAX_FILE_MB,
    MAX_LOG_RETENTION_DAYS, MAX_MERGE_SIDE_BY_SIDE_MIN_WIDTH, MAX_MOUSE_SCROLL_LINES,
    MAX_SCRIPT_TIMEOUT_MS, MAX_SCRIPT_TOTAL_TIMEOUT_S, MCP_DEFAULT_PORT, MIB_OR_GIB,
    MINUTES_OR_HOURS, MIN_BLAME_WIDTH, MIN_COMMITS_PER_LAZY_PAGE, MIN_EDITOR_TAB_SIZE,
    MIN_INITIAL_COMMITS_IN_GRAPH, MIN_MERGE_SIDE_BY_SIDE_MIN_WIDTH, MIN_MOUSE_SCROLL_LINES,
    SCRIPT_TIMEOUT_UNITS, SCRIPT_TOTAL_TIMEOUT_UNITS, THEME_SYSTEM,
};
pub use session_name::{
    default_session_name, default_session_name_from_root, list_session_port_files,
    sanitize_session_name, MAX_SESSION_NAME_LEN,
};
pub use settings::{env_var_for, Overrides, Settings, Source, ENV_PREFIX, LIVE_KEYS};

use std::path::Path;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("toml parse error: {0}")]
    Parse(#[from] toml::de::Error),
    #[error("toml serialize error: {0}")]
    Serialize(#[from] toml::ser::Error),
    #[error("no such config key: {0}")]
    UnknownKey(String),
    #[error("{key} cannot be set from the command line (it is a {kind}); edit config.toml")]
    UnsettableKey { key: String, kind: &'static str },
    #[error("invalid value for {key}: {value:?} ({expected})")]
    BadValue {
        key: String,
        value: String,
        expected: String,
    },
    #[error("config could not be represented as json: {0}")]
    Json(#[from] serde_json::Error),
}

#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct Config {
    #[serde(default)]
    pub general: GeneralConfig,
    #[serde(default)]
    pub ui: UiConfig,
    #[serde(default)]
    pub credentials: CredentialsConfig,
    #[serde(default)]
    pub ssh: SshConfig,
    /// Named commit identities; `general.active_profile` picks one.
    #[serde(default)]
    pub profiles: Vec<Profile>,
    /// GitHub/GitLab instances, hosted or self-managed. See
    /// [`Integration`] - the shape is a contract with
    /// `gitpoulpi-hosting`, and `token_ref` is a key into
    /// `gitpoulpi-secrets`, never a token.
    #[serde(default)]
    pub integrations: Vec<Integration>,
    #[serde(default)]
    pub git: GitConfig,
    #[serde(default)]
    pub hosting: Vec<HostingInstance>,
    #[serde(default)]
    pub mcp: McpConfig,
    #[serde(default)]
    pub ai: AiConfig,
    #[serde(default)]
    pub session: SessionConfig,
    /// The built-in editor and diff text settings: `editor.*`.
    #[serde(default)]
    pub editor: EditorConfig,
    /// The merge conflict editor's layout: `merge.*`.
    #[serde(default)]
    pub merge: MergeConfig,
    /// Where repositories live: `repos.clone_root`.
    #[serde(default)]
    pub repos: ReposConfig,
    /// `[keybindings]`: strings only, interpreted by the TUI. See
    /// [`KeybindingsConfig`].
    #[serde(default)]
    pub keybindings: KeybindingsConfig,
    /// The application log: `log.level`, `log.retention_days`.
    #[serde(default)]
    pub log: LogConfig,
    /// The workspace search: `search.commit_limit`.
    #[serde(default)]
    pub search: SearchConfig,
    /// History rewrite: `rewrite.protected_branches`.
    #[serde(default)]
    pub rewrite: RewriteConfig,
    /// Looking for a newer release: `updates.mode`, `updates.channel`.
    #[serde(default)]
    pub updates: UpdatesConfig,
    /// What counts as sensitive, and the staging guard: `privacy.*`.
    #[serde(default)]
    pub privacy: PrivacyConfig,
    /// Named lists of base branch patterns and the default one:
    /// `conflict_prevention.default_list`, `[conflict_prevention.lists.<name>]`.
    #[serde(default)]
    pub conflict_prevention: ConflictPreventionConfig,
    /// Per-installation look and names: `[installation]`. Left out of the
    /// file while empty. See [`installation`].
    #[serde(default, skip_serializing_if = "InstallationConfig::is_empty")]
    pub installation: InstallationConfig,
    /// User-defined named action sequences: `[[sequences]]`. Left out of the
    /// file while empty. See [`sequences`].
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub sequences: Vec<sequences::SequenceDef>,
}

/// What [`Config::profile_for`] needs of a workspace: its default profile id
/// and whether the repository at hand belongs to it. A struct of facts rather
/// than the workspace itself: workspaces live in `gitpoulpi-session`, which
/// depends on this crate, and the membership test (`same_repo_path`) is
/// there too.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WorkspaceDefault<'a> {
    /// The workspace's `profile`, `None` when it has no default.
    pub profile: Option<&'a str>,
    pub repo_is_member: bool,
}

impl Config {
    /// Loads just the `config.toml` layer. The full precedence ladder
    /// (defaults -> file -> `GITPOULPI_*` env -> CLI flags) is
    /// [`Settings::resolve`]; call that rather than this from anything
    /// user-facing, or the upper layers silently stop applying.
    ///
    /// When `path` is a `config.toml` with a sibling `tui.toml`, the keys that
    /// file owns (see [`split`]) come from it; without one the result is the
    /// `config.toml` alone, as before the split.
    pub fn load(path: &Path) -> Result<Self> {
        match split::read_merged(path)? {
            None => Ok(Self::default()),
            Some((doc, _)) => Ok(Self::from_document(doc)?.0),
        }
    }

    /// Parses a `config.toml` body and applies the legacy-key migrations
    /// (`UiConfig::migrate_legacy`), returning the raw document too for
    /// callers that need to know what the file itself said. Every loader
    /// goes through here, so no consumer ever sees a pre-migration value.
    pub fn from_toml_str(raw: &str) -> Result<(Self, toml::Value)> {
        Self::from_document(toml::from_str(raw)?)
    }

    /// [`Self::from_toml_str`] for a document already parsed (and, when a
    /// `tui.toml` exists, already merged).
    pub(crate) fn from_document(doc: toml::Value) -> Result<(Self, toml::Value)> {
        let mut migrated = doc.clone();
        AiConfig::migrate_document(&mut migrated);
        let mut config: Config = migrated.try_into()?;
        config.ui.migrate_legacy(doc.get("ui"));
        config.normalize_integrations();
        config.ai.migrate_legacy(doc.get("ai"));
        config.ai.ensure_default_profile();
        Ok((config, doc))
    }

    /// Migrates pre-multi-provider `[[integrations]]` entries: a `github`
    /// entry pointing at a non-github.com host becomes `github_enterprise`,
    /// a `gitlab` entry pointing at a non-gitlab.com host becomes
    /// `gitlab_self_managed`. Idempotent; run by every loader.
    pub fn normalize_integrations(&mut self) {
        for integration in &mut self.integrations {
            integration.normalize_kind();
        }
    }

    pub fn save(&self, path: &Path) -> Result<()> {
        if let Some(parent) = path.parent() {
            secure_fs::create_private_dir_all(parent)?;
        }
        let raw = toml::to_string_pretty(self)?;
        secure_fs::write_private(path, raw.as_bytes())?;
        Ok(())
    }

    /// The profile `general.active_profile` names, if it names one.
    ///
    /// An id pointing at nothing is deliberately not an error: profiles
    /// are a list the user edits, and deleting the active one must not
    /// make the config unloadable - it falls back to whatever
    /// `git config user.*` already says.
    pub fn active_profile(&self) -> Option<&Profile> {
        self.profiles
            .iter()
            .find(|p| p.id == self.general.active_profile)
    }

    /// The profile commits are made as in a repository, given the workspace
    /// it is being worked on from (`None` = no workspace).
    ///
    /// | Situation | Profile |
    /// |---|---|
    /// | the workspace names a profile that exists and the repository is in it | the workspace's |
    /// | anything else: no workspace, a repository outside it, no profile set, a profile id naming nothing | [`active_profile`](Self::active_profile) |
    ///
    /// Pure and total: a workspace whose profile was deleted must not break
    /// committing, it just stops applying. Which workspace is active is the
    /// caller's to say (the TUI passes its active workspace; the CLI and MCP
    /// take `--workspace`/`GITPOULPI_WORKSPACE` and never guess from
    /// membership, since a repository can be in several workspaces).
    pub fn profile_for(&self, workspace: Option<WorkspaceDefault<'_>>) -> Option<&Profile> {
        workspace
            .filter(|w| w.repo_is_member)
            .and_then(|w| w.profile)
            .and_then(|id| self.profiles.iter().find(|p| p.id == id))
            .or_else(|| self.active_profile())
    }

    /// Reads a dotted key (`general.default_backend`) as JSON.
    ///
    /// Routed through `serde_json` so the key space *is* the schema: a
    /// field added to any section is readable the moment it exists.
    pub fn get(&self, key: &str) -> Result<serde_json::Value> {
        let root = serde_json::to_value(self)?;
        let mut cursor = &root;
        for segment in key.split('.') {
            cursor = cursor
                .get(segment)
                .ok_or_else(|| Error::UnknownKey(key.to_string()))?;
        }
        Ok(cursor.clone())
    }

    /// Sets a dotted key from the string a user typed at a shell.
    ///
    /// The target's *current* JSON type decides how `value` is parsed, and
    /// the whole document is then deserialized back into `Config`, so serde
    /// validates enums like `default_backend` itself instead of a second
    /// list of legal spellings. `self` is only replaced once that
    /// round-trip succeeds, so a rejected value leaves it untouched.
    pub fn set(&mut self, key: &str, value: &str) -> Result<()> {
        let mut root = serde_json::to_value(&*self)?;
        let segments: Vec<&str> = key.split('.').collect();
        let (last, parents) = segments
            .split_last()
            .expect("split always yields at least one segment");

        let mut cursor = &mut root;
        for segment in parents {
            cursor = cursor
                .get_mut(segment)
                .ok_or_else(|| Error::UnknownKey(key.to_string()))?;
        }
        let leaf = cursor
            .get_mut(*last)
            .ok_or_else(|| Error::UnknownKey(key.to_string()))?;
        *leaf = coerce(key, leaf, value)?;

        *self = serde_json::from_value(root).map_err(|err| Error::BadValue {
            key: key.to_string(),
            value: value.to_string(),
            expected: err.to_string(),
        })?;
        Ok(())
    }
}

/// Strings as a TOML array.
fn strings_value(v: &[String]) -> toml::Value {
    toml::Value::Array(v.iter().cloned().map(toml::Value::String).collect())
}

/// A name-to-width map as a TOML table.
fn widths_value(widths: &std::collections::BTreeMap<String, u16>) -> toml::Value {
    let mut table = toml::map::Map::new();
    for (name, w) in widths {
        table.insert(name.clone(), toml::Value::Integer(i64::from(*w)));
    }
    toml::Value::Table(table)
}

/// One load-modify-save of the interface keys `entries` (all homed in
/// `tui.toml`, see [`split`]), the shape of every column writer.
fn set_ui_entries_in_file(path: &Path, entries: Vec<(&'static str, toml::Value)>) -> Result<()> {
    let key = entries.first().map_or("ui", |(k, _)| *k).to_string();
    split::edit_config_document(
        path,
        &key,
        |document| {
            for (key, value) in entries {
                insert_dotted(document, key, value)?;
            }
            Ok(())
        },
        |err| Error::BadValue {
            key: key.clone(),
            value: "<columns>".to_string(),
            expected: err.to_string(),
        },
    )
    .map(|_| ())
}

/// Writes the three `[ui]` commit-column keys (`commit_column_columns_visible`,
/// `commit_column_columns_order`, `commit_column_columns_widths`) in one
/// load-modify-save, leaving every other key as the file had it. Interface
/// keys: written to `tui.toml` (see [`split`]).
pub fn set_commit_column_in_file(
    path: &Path,
    visible: &[String],
    order: &[String],
    widths: &std::collections::BTreeMap<String, u16>,
) -> Result<()> {
    set_ui_entries_in_file(
        path,
        vec![
            ("ui.commit_column_columns_visible", strings_value(visible)),
            ("ui.commit_column_columns_order", strings_value(order)),
            ("ui.commit_column_columns_widths", widths_value(widths)),
        ],
    )
}

/// Writes the two global `[ui]` Log-column keys (`log_columns_hidden`,
/// `log_columns_order`) and leaves the rest of the file as it was. The widths
/// are per repository (`gitpoulpi-session`'s `RepoState::log_columns`).
pub fn set_log_column_layout_in_file(
    path: &Path,
    hidden: &[String],
    order: &[String],
) -> Result<()> {
    set_ui_entries_in_file(
        path,
        vec![
            ("ui.log_columns_hidden", strings_value(hidden)),
            ("ui.log_columns_order", strings_value(order)),
        ],
    )
}

/// Writes the four `[ui]` Log-column keys (`log_columns_hidden`,
/// `log_columns_order`, `log_column_widths`, `log_column_widths_scroll`) in
/// one load-modify-save, leaving every other key as the file had it.
pub fn set_log_columns_in_file(
    path: &Path,
    hidden: &[String],
    order: &[String],
    widths: &std::collections::BTreeMap<String, u16>,
    scroll_widths: &std::collections::BTreeMap<String, u16>,
) -> Result<()> {
    set_ui_entries_in_file(
        path,
        vec![
            ("ui.log_columns_hidden", strings_value(hidden)),
            ("ui.log_columns_order", strings_value(order)),
            ("ui.log_column_widths", widths_value(widths)),
            ("ui.log_column_widths_scroll", widths_value(scroll_widths)),
        ],
    )
}

/// Writes `[ui] log_columns_layout` (`fit` or `scroll`) in one load-modify-save,
/// under the same write lock as the column widths so a toggle and a width
/// save, run from two threads, never lose each other's key.
pub fn set_log_columns_layout_in_file(path: &Path, layout: LogColumnsLayout) -> Result<()> {
    set_ui_entries_in_file(
        path,
        vec![(
            "ui.log_columns_layout",
            toml::Value::String(layout.as_str().to_string()),
        )],
    )
}

/// Writes `[ui] sidebar_sections_hidden` (the left-pane sections the user
/// hid, by name) in one load-modify-save, leaving every other key as the
/// file had it.
pub fn set_sidebar_sections_hidden_in_file(path: &Path, hidden: &[String]) -> Result<()> {
    set_ui_entries_in_file(
        path,
        vec![("ui.sidebar_sections_hidden", strings_value(hidden))],
    )
}

/// Sets one dotted key in the `config.toml` *document* at `path`, leaving
/// every other key exactly as the file had it, and returns the value as
/// stored.
///
/// Loading a [`Config`], calling [`Config::set`] and [`Config::save`] would
/// be wrong: `Config` is fully populated, so saving writes every key at
/// its default, turning a three-line file into a hundred-line one and
/// breaking `Settings::source_of`, which answers "did the file say this?"
/// by looking for the key in the file.
///
/// The value's type comes from the schema (via a defaults instance) and
/// the edited document is deserialized into `Config` before anything is
/// written, so an unknown key or an illegal value is refused with the
/// file untouched.
///
/// Comments are not preserved: the document is re-serialized from
/// `toml::Value`, which does not carry them.
pub fn set_in_file(path: &Path, key: &str, value: &str) -> Result<serde_json::Value> {
    // `keybindings.<ctx>.<action>` is a free-form map the schema defaults
    // cannot describe, so it has its own writer. No semantic validation:
    // the TUI checks the chords when it starts.
    if let Some(rest) = key.strip_prefix("keybindings.") {
        let (ctx, action) = rest
            .split_once('.')
            .ok_or_else(|| Error::UnknownKey(key.to_string()))?;
        let chords = keybindings::chords_from_cli(value);
        set_keybinding_in_file(path, ctx, action, &chords)?;
        return Ok(serde_json::json!(chords));
    }
    // The schema, not the file, decides how the string a user typed is
    // parsed -- a file that is silent about the key would otherwise leave
    // nothing to infer a type from.
    let schema_value = Config::default().get(key)?;
    let coerced = coerce(key, &schema_value, value)?;

    let config = split::edit_config_document(
        path,
        key,
        |document| {
            AiConfig::migrate_document(document);
            insert_dotted(document, key, json_to_toml(coerced))
        },
        |err| Error::BadValue {
            key: key.to_string(),
            value: value.to_string(),
            expected: err.to_string(),
        },
    )?;
    config.get(key)
}

/// Replaces a whole array-of-tables section (`[[profiles]]`,
/// `[[integrations]]`) in the `config.toml` *document* at `path`, leaving
/// every other key exactly as the file had it.
///
/// [`set_in_file`] deliberately refuses lists - a shell argument cannot
/// express a record - but the Preferences screen edits these sections as
/// whole lists, so it needs a door of its own. Same guarantees otherwise:
/// the edited document is deserialized into [`Config`] before anything is
/// written, so a malformed list leaves the file untouched, and keys the
/// caller did not mention survive.
///
/// An empty `items` removes the section rather than writing `key = []`,
/// so "I deleted my last integration" leaves a file that reads like one
/// that never had one.
pub fn set_list_in_file<T: serde::Serialize>(path: &Path, key: &str, items: &[T]) -> Result<()> {
    split::edit_config_document(
        path,
        key,
        |document| {
            AiConfig::migrate_document(document);
            if items.is_empty() {
                remove_dotted(document, key);
            } else {
                insert_dotted(document, key, toml::Value::try_from(items)?)?;
            }
            Ok(())
        },
        // Validated before writing, exactly like `set_in_file`.
        |err| Error::BadValue {
            key: key.to_string(),
            value: "<list>".to_string(),
            expected: err.to_string(),
        },
    )
    .map(|_| ())
}

/// Writes (or, with `None`, removes) `[[ui.open_in]]`, keeping everything else
/// in the file. Unlike [`set_list_in_file`], `Some(&[])` is written as
/// `open_in = []`: an empty list the user chose is not "not configured yet".
pub fn set_open_in_in_file(path: &Path, entries: Option<&[OpenInEntry]>) -> Result<()> {
    set_entries_in_file(path, "ui.open_in", entries)
}

/// Replaces the whole `[conflict_prevention]` section (the default list's name
/// and every named list) in the document at `path`, keeping everything else.
/// A section equal to the default is removed rather than written empty.
pub fn set_conflict_prevention_in_file(
    path: &Path,
    section: &ConflictPreventionConfig,
) -> Result<()> {
    split::edit_config_document(
        path,
        "conflict_prevention",
        |document| {
            if section.is_default() {
                remove_dotted(document, "conflict_prevention");
                Ok(())
            } else {
                insert_dotted(
                    document,
                    "conflict_prevention",
                    toml::Value::try_from(section)?,
                )
            }
        },
        |err| Error::BadValue {
            key: "conflict_prevention".to_string(),
            value: "<section>".to_string(),
            expected: err.to_string(),
        },
    )
    .map(|_| ())
}

/// [`set_open_in_in_file`] for `[[ui.image_editors]]`.
pub fn set_image_editors_in_file(path: &Path, entries: Option<&[OpenInEntry]>) -> Result<()> {
    set_entries_in_file(path, "ui.image_editors", entries)
}

fn set_entries_in_file(path: &Path, key: &str, entries: Option<&[OpenInEntry]>) -> Result<()> {
    split::edit_config_document(
        path,
        key,
        |document| {
            match entries {
                None => remove_dotted(document, key),
                Some(items) => insert_dotted(document, key, toml::Value::try_from(items)?)?,
            }
            Ok(())
        },
        |err| Error::BadValue {
            key: key.to_string(),
            value: "<list>".to_string(),
            expected: err.to_string(),
        },
    )
    .map(|_| ())
}

/// Removes a dotted key, and any table it leaves empty, so deleting the
/// last `[[ai.profiles]]` entry does not leave a bare `[ai]` behind.
fn remove_dotted(document: &mut toml::Value, key: &str) {
    fn go(value: &mut toml::Value, segments: &[&str]) {
        let Some(table) = value.as_table_mut() else {
            return;
        };
        match segments {
            [] => {}
            [last] => {
                table.remove(*last);
            }
            [first, rest @ ..] => {
                if let Some(child) = table.get_mut(*first) {
                    go(child, rest);
                    if child.as_table().is_some_and(|t| t.is_empty()) {
                        table.remove(*first);
                    }
                }
            }
        }
    }
    let segments: Vec<&str> = key.split('.').collect();
    go(document, &segments);
}

/// Walks (creating as it goes) the tables named by all but the last
/// segment of `key`, then sets the leaf.
fn insert_dotted(document: &mut toml::Value, key: &str, value: toml::Value) -> Result<()> {
    let segments: Vec<&str> = key.split('.').collect();
    let (last, parents) = segments
        .split_last()
        .expect("split always yields at least one segment");

    let mut cursor = document;
    for segment in parents {
        let table = cursor.as_table_mut().ok_or_else(|| Error::UnsettableKey {
            key: key.to_string(),
            kind: "value",
        })?;
        cursor = table
            .entry(*segment)
            .or_insert_with(|| toml::Value::Table(toml::map::Map::new()));
    }
    let table = cursor.as_table_mut().ok_or_else(|| Error::UnsettableKey {
        key: key.to_string(),
        kind: "value",
    })?;
    table.insert(last.to_string(), value);
    Ok(())
}

/// `coerce` only ever produces a bool, an integer or a string, so this
/// covers exactly those.
fn json_to_toml(value: serde_json::Value) -> toml::Value {
    match value {
        serde_json::Value::Bool(b) => toml::Value::Boolean(b),
        serde_json::Value::Number(n) => n
            .as_i64()
            .map(toml::Value::Integer)
            .unwrap_or_else(|| toml::Value::String(n.to_string())),
        other => toml::Value::String(other.as_str().unwrap_or_default().to_string()),
    }
}

/// Parses `value` into whatever JSON type `current` already is.
///
/// Tables and arrays are refused outright: `[[hosting]]` is a list of
/// records, and pretending a shell argument could express one would
/// produce a config file the user did not mean.
fn coerce(key: &str, current: &serde_json::Value, value: &str) -> Result<serde_json::Value> {
    use serde_json::Value;
    let bad = |expected: &str| Error::BadValue {
        key: key.to_string(),
        value: value.to_string(),
        expected: expected.to_string(),
    };
    match current {
        Value::Bool(_) => value
            .parse::<bool>()
            .map(Value::Bool)
            .map_err(|_| bad("expected true or false")),
        Value::Number(_) => value
            .parse::<i64>()
            .map(Value::from)
            .map_err(|_| bad("expected a whole number")),
        Value::String(_) | Value::Null => Ok(Value::String(value.to_string())),
        Value::Array(_) => Err(Error::UnsettableKey {
            key: key.to_string(),
            kind: "list",
        }),
        Value::Object(_) => Err(Error::UnsettableKey {
            key: key.to_string(),
            kind: "table",
        }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_dotted_key_reads_back_the_value_in_the_schema() {
        let config = Config::default();
        assert_eq!(
            config.get("general.default_backend").unwrap(),
            serde_json::json!("git2")
        );
        assert_eq!(
            config.get("ui.mouse_enabled").unwrap(),
            serde_json::json!(true)
        );
    }

    #[test]
    fn a_dotted_key_can_name_a_whole_section() {
        let config = Config::default();
        let ui = config.get("ui").unwrap();
        assert_eq!(ui["theme"], serde_json::json!("system"));
    }

    #[test]
    fn an_unknown_key_is_rejected_rather_than_silently_ignored() {
        let mut config = Config::default();
        assert!(config.get("general.no_such_key").is_err());
        assert!(config.set("general.no_such_key", "1").is_err());
        assert!(config.set("nosuchsection.key", "1").is_err());
    }

    #[test]
    fn setting_an_enum_validates_against_the_schema_not_a_second_list() {
        let mut config = Config::default();
        config.set("general.default_backend", "system").unwrap();
        assert_eq!(config.general.default_backend, DefaultBackend::System);

        let err = config
            .set("general.default_backend", "mercurial")
            .unwrap_err();
        assert!(err.to_string().contains("mercurial"), "{err}");
        // Rejected values must not half-apply.
        assert_eq!(config.general.default_backend, DefaultBackend::System);
    }

    #[test]
    fn a_boolean_key_takes_true_and_false_and_nothing_else() {
        let mut config = Config::default();
        config.set("ui.mouse_enabled", "false").unwrap();
        assert!(!config.ui.mouse_enabled);
        assert!(config.set("ui.mouse_enabled", "no").is_err());
        assert!(!config.ui.mouse_enabled);
    }

    #[test]
    fn lists_and_tables_are_refused_with_a_reason() {
        let mut config = Config::default();
        assert!(matches!(
            config.set("hosting", "github"),
            Err(Error::UnsettableKey { kind: "list", .. })
        ));
        assert!(matches!(
            config.set("ui", "dark"),
            Err(Error::UnsettableKey { kind: "table", .. })
        ));
    }

    /// `set_in_file` must leave the file sparse. A fully-populated
    /// rewrite would make every default look like a deliberate choice,
    /// and `Settings::source_of` - which answers "did the file say
    /// this?" by looking - would then report `file` for everything.
    #[test]
    fn setting_one_key_does_not_write_out_the_whole_schema() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("config.toml");

        let stored = set_in_file(&path, "general.default_backend", "system").unwrap();
        assert_eq!(stored, serde_json::json!("system"));

        let raw = std::fs::read_to_string(&path).unwrap();
        let document: toml::Value = toml::from_str(&raw).unwrap();
        assert!(document.get("general").is_some(), "{raw}");
        assert!(document.get("ui").is_none(), "{raw}");
        assert!(document.get("mcp").is_none(), "{raw}");
    }

    #[test]
    fn setting_a_second_key_keeps_the_first_and_creates_missing_tables() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("config.toml");
        set_in_file(&path, "general.default_backend", "system").unwrap();
        set_in_file(&path, "ui.mouse_enabled", "false").unwrap();

        let config = Config::load(&path).unwrap();
        assert_eq!(config.general.default_backend, DefaultBackend::System);
        assert!(!config.ui.mouse_enabled);
    }

    #[test]
    fn set_in_file_refuses_a_bad_value_without_touching_the_file() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("config.toml");
        set_in_file(&path, "general.default_backend", "system").unwrap();
        let before = std::fs::read_to_string(&path).unwrap();

        assert!(set_in_file(&path, "general.default_backend", "mercurial").is_err());
        assert!(set_in_file(&path, "general.no_such_key", "1").is_err());
        assert_eq!(std::fs::read_to_string(&path).unwrap(), before);
    }

    #[test]
    fn commit_column_keys_round_trip_apart_from_the_logs() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("config.toml");
        set_log_columns_in_file(
            &path,
            &["sha".to_string()],
            &["graph".to_string()],
            &Default::default(),
            &Default::default(),
        )
        .unwrap();
        let widths = [("author".to_string(), 12u16)].into_iter().collect();
        set_commit_column_in_file(
            &path,
            &["ref".to_string(), "message".to_string()],
            &["sha".to_string(), "ref".to_string()],
            &widths,
        )
        .unwrap();
        let config = Config::load(&path).unwrap();
        assert_eq!(
            config.ui.commit_column_columns_visible,
            vec!["ref", "message"]
        );
        assert_eq!(config.ui.commit_column_columns_order, vec!["sha", "ref"]);
        assert_eq!(
            config.ui.commit_column_columns_widths.get("author"),
            Some(&12)
        );
        assert_eq!(config.ui.log_columns_hidden, vec!["sha"]);
        assert!(config.ui.log_column_widths.is_empty());
    }

    #[test]
    fn sidebar_sections_hidden_round_trips_through_the_file() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("config.toml");
        std::fs::write(&path, "[general]\nauto_prune = true\n").unwrap();
        set_sidebar_sections_hidden_in_file(&path, &["tags".into(), "issues".into()]).unwrap();
        let config = Config::load(&path).unwrap();
        assert_eq!(config.ui.sidebar_sections_hidden, vec!["tags", "issues"]);
        assert!(config.general.auto_prune);
        set_sidebar_sections_hidden_in_file(&path, &[]).unwrap();
        assert!(Config::load(&path)
            .unwrap()
            .ui
            .sidebar_sections_hidden
            .is_empty());
    }

    #[test]
    fn log_columns_round_trip_through_the_file() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("config.toml");
        set_in_file(&path, "general.default_backend", "system").unwrap();
        let mut widths = std::collections::BTreeMap::new();
        widths.insert("author".to_string(), 21u16);
        set_log_columns_in_file(
            &path,
            &["sha".to_string(), "date".to_string()],
            &["graph".to_string(), "ref".to_string()],
            &widths,
            &[("message".to_string(), 55u16)].into_iter().collect(),
        )
        .unwrap();
        let config = Config::load(&path).unwrap();
        assert_eq!(config.ui.log_columns_hidden, vec!["sha", "date"]);
        assert_eq!(config.ui.log_columns_order, vec!["graph", "ref"]);
        assert_eq!(config.ui.log_column_widths.get("author"), Some(&21));
        // The scroll layout's widths are their own map, `message` included.
        assert_eq!(config.ui.log_column_widths_scroll.get("message"), Some(&55));
        assert_eq!(config.ui.log_columns_layout, LogColumnsLayout::Fit);
        assert_eq!(config.general.default_backend, DefaultBackend::System);

        // The layout key is written apart and keeps the widths.
        set_log_columns_layout_in_file(&path, LogColumnsLayout::Scroll).unwrap();
        let config = Config::load(&path).unwrap();
        assert_eq!(config.ui.log_columns_layout, LogColumnsLayout::Scroll);
        assert_eq!(config.ui.log_column_widths_scroll.get("message"), Some(&55));
    }

    /// The external-tool templates: empty editor/diff tool by default, a
    /// per-platform explorer, all settable and live.
    #[test]
    fn external_tool_templates_default_sensibly_and_round_trip() {
        let defaults = Config::default().general;
        assert_eq!(defaults.external_editor, "");
        assert_eq!(defaults.external_diff_tool, "");
        assert!(
            defaults.file_explorer.contains("{path}") || defaults.file_explorer.contains("{dir}")
        );
        if cfg!(windows) {
            assert_eq!(defaults.file_explorer, "explorer.exe /select,\"{path}\"");
        }

        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("config.toml");
        set_in_file(&path, "general.external_editor", "code -r {file}").unwrap();
        set_in_file(&path, "general.external_diff_tool", "meld {local} {remote}").unwrap();
        set_in_file(&path, "general.file_explorer", "xdg-open \"{dir}\"").unwrap();
        let loaded = Config::load(&path).unwrap();
        assert_eq!(loaded.general.external_editor, "code -r {file}");
        assert_eq!(loaded.general.external_diff_tool, "meld {local} {remote}");
        assert_eq!(loaded.general.file_explorer, "xdg-open \"{dir}\"");
        for key in [
            "general.external_editor",
            "general.external_diff_tool",
            "general.file_explorer",
        ] {
            assert!(LIVE_KEYS.contains(&key), "{key} must be a live key");
        }
    }

    /// Hand-written keys this command was not asked about have to survive
    /// it, or `config set` becomes something a user cannot safely run on
    /// a file they maintain themselves.
    #[test]
    fn set_in_file_leaves_keys_it_was_not_asked_about_alone() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("config.toml");
        std::fs::write(
            &path,
            "[general]\nauto_stash = \"always\"\n\n[[hosting]]\nprovider = \"gitlab\"\n\
             base_url = \"https://gitlab.example.com\"\nauth = \"credential_helper\"\n",
        )
        .unwrap();

        set_in_file(&path, "general.default_backend", "system").unwrap();

        let config = Config::load(&path).unwrap();
        assert_eq!(config.general.auto_stash, AutoStashMode::Always);
        assert_eq!(config.hosting.len(), 1);
        assert_eq!(config.hosting[0].provider, "gitlab");
        assert_eq!(config.general.default_backend, DefaultBackend::System);
    }

    /// `set_list_in_file` is the door `set_in_file` deliberately does
    /// not open, and it has to keep the same promise: sparse file, other
    /// keys untouched.
    #[test]
    fn writing_a_list_section_leaves_every_other_key_alone() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("config.toml");
        set_in_file(&path, "general.default_backend", "system").unwrap();

        let integrations = vec![Integration::new(
            "my-gitlab",
            IntegrationKind::GitlabSelfManaged,
            "https://gitlab.example.com",
        )];
        set_list_in_file(&path, "integrations", &integrations).unwrap();

        let config = Config::load(&path).unwrap();
        assert_eq!(config.general.default_backend, DefaultBackend::System);
        assert_eq!(config.integrations, integrations);
        assert_eq!(config.integrations[0].token_ref, "integration:my-gitlab");

        let raw = std::fs::read_to_string(&path).unwrap();
        assert!(raw.contains("[[integrations]]"), "{raw}");
        // Still sparse: sections nobody set stay out of the file.
        assert!(!raw.contains("[mcp]"), "{raw}");
    }

    /// Emptying a list removes the section rather than leaving
    /// `integrations = []`, so a file that once had one reads like one
    /// that never did.
    #[test]
    fn emptying_a_list_removes_its_section() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("config.toml");
        set_list_in_file(&path, "profiles", &[Profile::new("work", "Work")]).unwrap();
        assert!(std::fs::read_to_string(&path).unwrap().contains("profiles"));

        set_list_in_file(&path, "profiles", &[] as &[Profile]).unwrap();
        let raw = std::fs::read_to_string(&path).unwrap();
        assert!(!raw.contains("profiles"), "{raw}");
        assert!(Config::load(&path).unwrap().profiles.is_empty());
    }

    /// The active profile pointing at nothing must stay loadable: the
    /// list is user-edited, and deleting the active entry is normal.
    #[test]
    fn an_active_profile_naming_no_profile_is_not_an_error() {
        let mut config = Config::default();
        config.profiles.push(Profile::new("work", "Work"));
        config.general.active_profile = "home".into();
        assert!(config.active_profile().is_none());
        config.general.active_profile = "work".into();
        assert_eq!(config.active_profile().unwrap().name, "Work");
    }

    fn two_profiles() -> Config {
        let mut config = Config::default();
        config.profiles.push(Profile::new("work", "Work"));
        config.profiles.push(Profile::new("home", "Home"));
        config
    }

    fn scope(profile: Option<&str>, member: bool) -> Option<WorkspaceDefault<'_>> {
        Some(WorkspaceDefault {
            profile,
            repo_is_member: member,
        })
    }

    fn id_of(p: Option<&Profile>) -> Option<&str> {
        p.map(|p| p.id.as_str())
    }

    /// Every row of the table in `profile_for`'s doc.
    #[test]
    fn profile_for_follows_its_table() {
        let mut config = two_profiles();
        config.general.active_profile = "home".into();
        // The workspace names a profile that exists, repository inside.
        assert_eq!(
            id_of(config.profile_for(scope(Some("work"), true))),
            Some("work")
        );
        // Repository outside the workspace: today's behavior.
        assert_eq!(
            id_of(config.profile_for(scope(Some("work"), false))),
            Some("home")
        );
        // The workspace names no profile.
        assert_eq!(id_of(config.profile_for(scope(None, true))), Some("home"));
        // The profile id points at nothing: the active profile, no error.
        assert_eq!(
            id_of(config.profile_for(scope(Some("deleted"), true))),
            Some("home")
        );
        // No workspace at all.
        assert_eq!(id_of(config.profile_for(None)), Some("home"));
    }

    #[test]
    fn profile_for_is_none_when_nothing_names_a_profile() {
        let mut config = two_profiles();
        // No active profile: only a valid workspace profile applies.
        assert_eq!(id_of(config.profile_for(None)), None);
        assert_eq!(id_of(config.profile_for(scope(None, true))), None);
        assert_eq!(id_of(config.profile_for(scope(Some("ghost"), true))), None);
        assert_eq!(
            id_of(config.profile_for(scope(Some("work"), true))),
            Some("work")
        );
        // An active profile naming nothing, and a workspace profile that
        // does not apply: no profile, never a panic.
        config.general.active_profile = "gone".into();
        assert_eq!(id_of(config.profile_for(scope(Some("work"), false))), None);
    }

    /// Round-tripping through TOML must not lose or rename anything, or
    /// `config set` would quietly rewrite keys the user did not touch.
    #[test]
    fn save_then_load_preserves_every_section() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("nested").join("config.toml");
        let mut config = Config::default();
        config.set("general.default_backend", "system").unwrap();
        config.set("ui.mouse_enabled", "false").unwrap();
        config.set("mcp.bind", "127.0.0.1:7000").unwrap();
        config.save(&path).unwrap();

        let back = Config::load(&path).unwrap();
        assert_eq!(back.general.default_backend, DefaultBackend::System);
        assert!(!back.ui.mouse_enabled);
        assert_eq!(back.mcp.bind, "127.0.0.1:7000");
    }
}
