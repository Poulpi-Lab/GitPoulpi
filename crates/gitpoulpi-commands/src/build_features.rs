//! What this build of the command layer was compiled with (custom builds,
//! the custom-builds design). The CLI, and through it the
//! MCP server, read these to describe themselves (`--version`,
//! `daemon status`) and to report "not built" errors in a stable shape.

/// A feature a custom build can leave out.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Feature {
    Ai,
    Hosting,
    Mcp,
    Lfs,
    Worktree,
    Submodule,
}

impl Feature {
    pub const ALL: [Feature; 6] = [
        Feature::Ai,
        Feature::Hosting,
        Feature::Mcp,
        Feature::Lfs,
        Feature::Worktree,
        Feature::Submodule,
    ];

    /// Whether this binary was built with the feature.
    pub const fn built(self) -> bool {
        match self {
            Feature::Ai => AI,
            Feature::Hosting => HOSTING,
            Feature::Mcp => MCP,
            Feature::Lfs => LFS,
            Feature::Worktree => WORKTREE,
            Feature::Submodule => SUBMODULE,
        }
    }

    /// The cargo feature name (also the `NotBuilt` payload).
    pub const fn name(self) -> &'static str {
        match self {
            Feature::Ai => "ai",
            Feature::Hosting => "hosting",
            Feature::Mcp => "mcp",
            Feature::Lfs => "lfs",
            Feature::Worktree => "worktree",
            Feature::Submodule => "submodule",
        }
    }
}

pub const AI: bool = cfg!(feature = "ai");
pub const HOSTING: bool = cfg!(feature = "hosting");
pub const GITHUB: bool = cfg!(feature = "github");
pub const GITLAB: bool = cfg!(feature = "gitlab");
pub const AZURE_DEVOPS: bool = cfg!(feature = "azure-devops");
pub const BITBUCKET: bool = cfg!(feature = "bitbucket");
pub const JIRA: bool = cfg!(feature = "jira");
pub const TRELLO: bool = cfg!(feature = "trello");
pub const GITEA: bool = cfg!(feature = "gitea");
pub const LINEAR: bool = cfg!(feature = "linear");
pub const YOUTRACK: bool = cfg!(feature = "youtrack");
pub const REDMINE: bool = cfg!(feature = "redmine");
pub const GERRIT: bool = cfg!(feature = "gerrit");
pub const LOCAL_ISSUES: bool = cfg!(feature = "local-issues");
pub const MCP: bool = cfg!(feature = "mcp");
pub const LFS: bool = cfg!(feature = "lfs");
pub const WORKTREE: bool = cfg!(feature = "worktree");
pub const SUBMODULE: bool = cfg!(feature = "submodule");
pub const AVATAR_FETCH: bool = cfg!(feature = "avatar-fetch");
pub const IMAGE_VIEW: bool = cfg!(feature = "image-view");
pub const RADAR: bool = cfg!(all(feature = "radar", feature = "hosting"));
pub const MERMAID: bool = cfg!(feature = "mermaid");
pub const BOARD: bool = cfg!(all(feature = "board", feature = "hosting"));
pub const REVIEW: bool = cfg!(all(feature = "review", feature = "hosting"));
pub const HISTORY_REWRITE: bool = cfg!(feature = "history-rewrite");
pub const GLOBAL_SEARCH: bool = cfg!(feature = "global-search");
pub const WORKSPACES: bool = cfg!(feature = "workspaces");
pub const REWRITE_SCRIPTS: bool = cfg!(feature = "rewrite-scripts");

/// The user-facing feature table, in `--version` order.
pub const FLAGS: [(&str, bool); 27] = [
    ("ai", AI),
    ("github", GITHUB),
    ("gitlab", GITLAB),
    ("azure-devops", AZURE_DEVOPS),
    ("bitbucket", BITBUCKET),
    ("jira", JIRA),
    ("trello", TRELLO),
    ("gitea", GITEA),
    ("linear", LINEAR),
    ("youtrack", YOUTRACK),
    ("redmine", REDMINE),
    ("gerrit", GERRIT),
    ("local-issues", LOCAL_ISSUES),
    ("mcp", MCP),
    ("lfs", LFS),
    ("worktree", WORKTREE),
    ("submodule", SUBMODULE),
    ("avatar-fetch", AVATAR_FETCH),
    ("image-view", IMAGE_VIEW),
    ("radar", RADAR),
    ("mermaid", MERMAID),
    ("board", BOARD),
    ("review", REVIEW),
    ("history-rewrite", HISTORY_REWRITE),
    ("global-search", GLOBAL_SEARCH),
    ("workspaces", WORKSPACES),
    ("rewrite-scripts", REWRITE_SCRIPTS),
];

/// Whether this build has the feature an AI action belongs to: the two
/// history rewrite actions are the History workbench's.
pub fn ai_action_offered(action: gitpoulpi_config::AiAction) -> bool {
    use gitpoulpi_config::AiAction as A;
    !matches!(action, A::RewriteScript | A::RewriteRules) || HISTORY_REWRITE
}

/// `ai.enabled` in the config AND the feature built.
pub fn ai_available(cfg: &gitpoulpi_config::Config) -> bool {
    AI && cfg.ai.enabled
}

/// `git.lfs_enabled` in the config AND the feature built.
pub fn lfs_available(cfg: &gitpoulpi_config::Config) -> bool {
    LFS && cfg.git.lfs_enabled
}

/// The language codes built into this binary (English first; the layout
/// test pseudo-locale is never listed).
pub fn languages() -> Vec<&'static str> {
    gitpoulpi_i18n::Lang::ALL
        .iter()
        .filter(|l| l.is_built() && **l != gitpoulpi_i18n::Lang::Qps)
        .map(|l| l.code())
        .collect()
}

/// The `--version` long form of this binary.
pub fn long_version() -> String {
    gitpoulpi_config::theme::with_theme_line(gitpoulpi_ipc::build_info::describe(
        &FLAGS,
        &languages(),
    ))
}

/// [`long_version`], built once, for clap's `long_version` (which wants a
/// `&'static str`).
pub fn long_version_static() -> &'static str {
    static TEXT: std::sync::OnceLock<String> = std::sync::OnceLock::new();
    TEXT.get_or_init(long_version)
}

/// `{"features": [...], "languages": [...]}` for `daemon status --json`.
pub fn status_fields() -> serde_json::Value {
    serde_json::json!({
        "features": gitpoulpi_ipc::build_info::enabled(&FLAGS),
        "languages": languages(),
        "themes": gitpoulpi_config::theme::theme_set(),
    })
}

/// The value to print under `"error"` for a command failure in `--json`
/// mode: `{"kind":"not_built","feature":"lfs"}` when the daemon refused
/// because the feature is not built, else the plain message.
pub fn error_value(message: &str) -> serde_json::Value {
    match not_built_feature(message) {
        Some(feature) => serde_json::json!({ "kind": "not_built", "feature": feature }),
        None => serde_json::Value::String(message.to_string()),
    }
}

/// The whole `--json` failure object: `{"error": <error_value>}` plus, when the
/// daemon sent the facts of a backend failure, `"detail"` (operation, the
/// redacted command line, exit code, stderr tail, libgit2 class/code).
pub fn error_object(message: &str) -> serde_json::Value {
    let mut object = serde_json::json!({ "error": error_value(message) });
    if let Some(detail) = crate::client::error_detail_for(message) {
        object["detail"] = serde_json::to_value(detail).unwrap_or_default();
    }
    object
}

/// The feature a `NotBuilt` error message names, if `message` is one
/// (`gitpoulpi_core::Error::NotBuilt`'s `Display`, as it crosses the wire).
pub fn not_built_feature(message: &str) -> Option<&'static str> {
    let name = message.strip_suffix(" is not built into this binary")?;
    gitpoulpi_core::BUILD_FEATURES
        .iter()
        .find(|f| **f == name)
        .copied()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn not_built_errors_become_structured_json() {
        let message = gitpoulpi_core::Error::NotBuilt { feature: "lfs" }.to_string();
        assert_eq!(
            error_value(&message),
            serde_json::json!({ "kind": "not_built", "feature": "lfs" })
        );
        assert_eq!(
            error_value("boom"),
            serde_json::Value::String("boom".into())
        );
        assert_eq!(not_built_feature("zzz is not built into this binary"), None);
    }

    #[test]
    fn the_default_build_lists_every_feature_and_language() {
        if !cfg!(feature = "full") {
            return;
        }
        let text = long_version();
        assert!(text.contains(
            "features: ai github gitlab azure-devops bitbucket jira trello gitea linear youtrack redmine gerrit local-issues mcp lfs worktree submodule avatar-fetch image-view radar mermaid board review history-rewrite global-search workspaces rewrite-scripts"
        ));
        assert!(text.starts_with(gitpoulpi_ipc::VERSION_STRING));
        assert!(!text.contains("custom build"));
        assert!(languages().len() >= 17);
    }

    #[test]
    fn the_feature_enum_matches_the_flags() {
        for f in Feature::ALL {
            assert!(gitpoulpi_core::BUILD_FEATURES.contains(&f.name()));
        }
    }
}
