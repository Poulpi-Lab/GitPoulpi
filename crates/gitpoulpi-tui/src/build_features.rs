//! What this build of the TUI was compiled with (custom builds, see
//! the custom-builds design), and the one filter every
//! registry (keymap, palette, Settings pages) applies to entries tagged
//! with a feature.
//!
//! A tag never removes an enum variant or a config key: a user's
//! `keybindings.toml` entry for an action this build lacks still parses, the
//! action is just not listed or run.
#![allow(dead_code)]

/// A feature a custom build can leave out.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Feature {
    Ai,
    Hosting,
    Mcp,
    Lfs,
    Worktree,
    Submodule,
    ImageView,
    Radar,
    Mermaid,
    Board,
    Review,
    Conversation,
    HistoryRewrite,
    GlobalSearch,
    Workspaces,
}

impl Feature {
    pub const ALL: [Feature; 15] = [
        Feature::Ai,
        Feature::Hosting,
        Feature::Mcp,
        Feature::Lfs,
        Feature::Worktree,
        Feature::Submodule,
        Feature::ImageView,
        Feature::Radar,
        Feature::Mermaid,
        Feature::Board,
        Feature::Review,
        Feature::Conversation,
        Feature::HistoryRewrite,
        Feature::GlobalSearch,
        Feature::Workspaces,
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
            Feature::ImageView => IMAGE_VIEW,
            Feature::Radar => RADAR,
            Feature::Mermaid => MERMAID,
            Feature::Board => BOARD,
            Feature::Review => REVIEW,
            Feature::Conversation => CONVERSATION,
            Feature::HistoryRewrite => HISTORY_REWRITE,
            Feature::GlobalSearch => GLOBAL_SEARCH,
            Feature::Workspaces => WORKSPACES,
        }
    }

    /// The cargo feature name.
    pub const fn name(self) -> &'static str {
        match self {
            Feature::Ai => "ai",
            Feature::Hosting => "hosting",
            Feature::Mcp => "mcp",
            Feature::Lfs => "lfs",
            Feature::Worktree => "worktree",
            Feature::Submodule => "submodule",
            Feature::ImageView => "image-view",
            Feature::Radar => "radar",
            Feature::Mermaid => "mermaid",
            Feature::Board => "board",
            Feature::Review => "review",
            Feature::Conversation => "conversation",
            Feature::HistoryRewrite => "history-rewrite",
            Feature::GlobalSearch => "global-search",
            Feature::Workspaces => "workspaces",
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
pub const CONVERSATION: bool = cfg!(all(feature = "conversation", feature = "hosting"));
pub const HISTORY_REWRITE: bool = cfg!(feature = "history-rewrite");
pub const GLOBAL_SEARCH: bool = cfg!(feature = "global-search");
pub const WORKSPACES: bool = cfg!(feature = "workspaces");
/// The embedded Lua runtime behind the history rewrite's Script rule.
pub const REWRITE_SCRIPTS: bool = cfg!(all(
    feature = "history-rewrite",
    feature = "rewrite-scripts"
));

/// The integration kinds this build has a client for: the GitHub family
/// with `github`, the GitLab family with `gitlab`. Pickers offer exactly
/// these.
pub const PROVIDER_KINDS: &[gitpoulpi_config::IntegrationKind] = {
    use gitpoulpi_config::IntegrationKind as K;
    if GITHUB && GITLAB {
        K::IMPLEMENTED
    } else if GITHUB {
        &[K::Github, K::GithubEnterprise]
    } else if GITLAB {
        &[K::Gitlab, K::GitlabSelfManaged]
    } else {
        &[]
    }
};

/// Whether this build has the client of an API family.
pub const fn family_built(family: gitpoulpi_config::ApiFamily) -> bool {
    use gitpoulpi_config::ApiFamily as F;
    match family {
        F::Github => GITHUB,
        F::Gitlab => GITLAB,
        F::AzureDevops => AZURE_DEVOPS,
        F::Bitbucket => BITBUCKET,
        F::Jira => JIRA,
        F::Trello => TRELLO,
        F::Gitea => GITEA,
        F::Linear => LINEAR,
        F::YouTrack => YOUTRACK,
        F::Redmine => REDMINE,
        F::Gerrit => GERRIT,
        F::LocalMarkdown => LOCAL_ISSUES,
    }
}

/// The kinds the Integrations page offers: `IMPLEMENTED` filtered by the
/// families this build has. Supersedes [`PROVIDER_KINDS`], which stays only
/// for its slice-shaped callers until the Integrations page moves over.
pub fn provider_kinds() -> Vec<gitpoulpi_config::IntegrationKind> {
    gitpoulpi_config::IntegrationKind::IMPLEMENTED
        .iter()
        .copied()
        .filter(|k| family_built(k.family()))
        .collect()
}

/// Whether this build has a client for `kind` (a profile of a kind that is
/// not built stays in the config but is never used).
pub fn kind_built(kind: gitpoulpi_config::IntegrationKind) -> bool {
    gitpoulpi_config::IntegrationKind::IMPLEMENTED.contains(&kind) && family_built(kind.family())
}

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
    ("conversation", CONVERSATION),
    ("history-rewrite", HISTORY_REWRITE),
    ("global-search", GLOBAL_SEARCH),
    ("workspaces", WORKSPACES),
];

/// `ai.enabled` in the config AND the feature built.
pub fn ai_available(cfg: &gitpoulpi_config::Config) -> bool {
    AI && cfg.ai.enabled
}

/// `git.lfs_enabled` in the config AND the feature built.
pub fn lfs_available(cfg: &gitpoulpi_config::Config) -> bool {
    LFS && cfg.git.lfs_enabled
}

/// Whether something tagged `requires` is offered, given which features
/// `built` reports. Untagged is always offered. Registries call
/// [`offered`]; tests call this with a fake table.
pub fn offered_with(requires: Option<Feature>, built: impl Fn(Feature) -> bool) -> bool {
    requires.is_none_or(built)
}

/// [`offered_with`] for this build.
pub fn offered(requires: Option<Feature>) -> bool {
    offered_with(requires, |f| {
        f.built() && (f != Feature::Radar || radar_enabled())
    })
}

/// `general.radar_enabled`, published like the other live settings: the
/// one switch every Radar surface (tab, palette entries, keys, background
/// fetches) reads through [`offered`] / [`radar_enabled`]. Tests get a
/// per-thread copy so parallel tests never see each other's value.
#[cfg(not(test))]
static RADAR_ENABLED: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(true);

#[cfg(test)]
thread_local! {
    static RADAR_ENABLED_TEST: std::cell::Cell<bool> = const { std::cell::Cell::new(true) };
}

/// This session's "Radar tab closed" (`session_store::radar_closed`),
/// published for the tab bar's paint walk, which has no session in hand.
#[cfg(not(test))]
static RADAR_CLOSED: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

#[cfg(test)]
thread_local! {
    static RADAR_CLOSED_TEST: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
}

pub fn install_radar_closed(closed: bool) {
    #[cfg(not(test))]
    RADAR_CLOSED.store(closed, std::sync::atomic::Ordering::Relaxed);
    #[cfg(test)]
    RADAR_CLOSED_TEST.with(|c| c.set(closed));
}

pub fn radar_closed() -> bool {
    #[cfg(not(test))]
    return RADAR_CLOSED.load(std::sync::atomic::Ordering::Relaxed);
    #[cfg(test)]
    return RADAR_CLOSED_TEST.with(std::cell::Cell::get);
}

/// This session's "Workspaces tab open" (`session_store::workspaces_open`),
/// published for the tab bar's paint walk, which has no session in hand.
#[cfg(not(test))]
static WORKSPACES_OPEN: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

#[cfg(test)]
thread_local! {
    static WORKSPACES_OPEN_TEST: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
}

pub fn install_workspaces_open(open: bool) {
    #[cfg(not(test))]
    WORKSPACES_OPEN.store(open, std::sync::atomic::Ordering::Relaxed);
    #[cfg(test)]
    WORKSPACES_OPEN_TEST.with(|c| c.set(open));
}

/// Whether the tab bar shows the Workspaces tab.
pub fn workspaces_tab_shown() -> bool {
    if !WORKSPACES {
        return false;
    }
    #[cfg(not(test))]
    return WORKSPACES_OPEN.load(std::sync::atomic::Ordering::Relaxed);
    #[cfg(test)]
    return WORKSPACES_OPEN_TEST.with(std::cell::Cell::get);
}

/// Which pinned tabs the last painted tab bar actually drew (radar,
/// workspaces). A tab is "shown" in the session's state yet not drawn when
/// the bar has no room for it; the `+` menu offers whatever is not drawn,
/// so a tab squeezed out of a narrow bar can still be reached.
#[cfg(not(test))]
static TABS_DRAWN: std::sync::atomic::AtomicU8 = std::sync::atomic::AtomicU8::new(0);

#[cfg(test)]
thread_local! {
    static TABS_DRAWN_TEST: std::cell::Cell<u8> = const { std::cell::Cell::new(0) };
}

pub fn install_tabs_drawn(radar: bool, workspaces: bool) {
    let bits = u8::from(radar) | (u8::from(workspaces) << 1);
    #[cfg(not(test))]
    TABS_DRAWN.store(bits, std::sync::atomic::Ordering::Relaxed);
    #[cfg(test)]
    TABS_DRAWN_TEST.with(|c| c.set(bits));
}

fn tabs_drawn() -> u8 {
    #[cfg(not(test))]
    return TABS_DRAWN.load(std::sync::atomic::Ordering::Relaxed);
    #[cfg(test)]
    return TABS_DRAWN_TEST.with(std::cell::Cell::get);
}

/// The `+` menu's Radar row: built, enabled, and its tab not on the bar.
pub fn plus_menu_offers_radar() -> bool {
    offered(Some(Feature::Radar)) && (radar_closed() || tabs_drawn() & 1 == 0)
}

/// The `+` menu's Workspaces row: built, and its tab not on the bar.
pub fn plus_menu_offers_workspaces() -> bool {
    WORKSPACES && (!workspaces_tab_shown() || tabs_drawn() & 2 == 0)
}

/// Whether the tab bar shows the Radar tab: built, enabled, not closed.
pub fn radar_tab_shown() -> bool {
    offered(Some(Feature::Radar)) && !radar_closed()
}

pub fn install_radar_enabled(on: bool) {
    #[cfg(not(test))]
    RADAR_ENABLED.store(on, std::sync::atomic::Ordering::Relaxed);
    #[cfg(test)]
    RADAR_ENABLED_TEST.with(|c| c.set(on));
}

/// Whether the user left the Radar on (the build having it is
/// [`Feature::built`]'s question).
pub fn radar_enabled() -> bool {
    #[cfg(not(test))]
    return RADAR_ENABLED.load(std::sync::atomic::Ordering::Relaxed);
    #[cfg(test)]
    return RADAR_ENABLED_TEST.with(std::cell::Cell::get);
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

#[cfg(test)]
mod tests {
    use super::*;

    /// `general.radar_enabled` off: nothing tagged `Radar` is offered
    /// (palette, keymap, tab), and the other features are untouched. Closing
    /// the tab alone hides the tab but not the commands that reopen it.
    #[test]
    fn the_radar_setting_hides_every_radar_surface() {
        install_radar_enabled(true);
        install_radar_closed(false);
        assert_eq!(offered(Some(Feature::Radar)), Feature::Radar.built());
        install_radar_enabled(false);
        assert!(!offered(Some(Feature::Radar)));
        assert!(!radar_tab_shown());
        assert!(offered(Some(Feature::Hosting)) == Feature::Hosting.built());
        install_radar_enabled(true);
        install_radar_closed(true);
        assert!(!radar_tab_shown(), "closed: no tab");
        assert_eq!(offered(Some(Feature::Radar)), Feature::Radar.built());
        install_radar_closed(false);
        assert_eq!(radar_tab_shown(), Feature::Radar.built());
    }

    #[cfg(all(feature = "radar", feature = "hosting"))]
    #[test]
    fn a_disabled_radar_does_not_open_and_so_starts_no_fetch() {
        install_radar_enabled(false);
        let mut slot = crate::radar::RadarSlot::default();
        slot.open("s");
        assert!(!slot.is_open(), "no view means no worker and no fetch");
        install_radar_enabled(true);
    }

    #[cfg(all(feature = "radar", feature = "hosting"))]
    #[test]
    fn the_palette_has_no_radar_entry_when_disabled() {
        let listed = || {
            crate::palette::PALETTE_COMMANDS
                .iter()
                .filter(|e| {
                    offered(e.requires) && offered(crate::palette::command_requires(&e.command))
                })
                .any(|e| matches!(e.command, crate::palette::PaletteCommand::Radar(_)))
        };
        install_radar_enabled(true);
        assert!(listed());
        install_radar_enabled(false);
        assert!(!listed());
        install_radar_enabled(true);
    }

    /// A fake build with only the listed features.
    fn only(on: &'static [Feature]) -> impl Fn(Feature) -> bool {
        move |f| on.contains(&f)
    }

    #[test]
    fn untagged_entries_are_always_offered() {
        assert!(offered_with(None, only(&[])));
    }

    #[test]
    fn tagged_entries_follow_the_build() {
        assert!(offered_with(Some(Feature::Ai), only(&[Feature::Ai])));
        assert!(!offered_with(Some(Feature::Ai), only(&[Feature::Lfs])));
        assert!(!offered_with(Some(Feature::Lfs), only(&[])));
    }

    #[test]
    fn the_default_build_offers_everything() {
        if cfg!(feature = "full") {
            assert!(Feature::ALL.iter().all(|f| f.built()));
            let text = long_version();
            assert!(text.contains(
                "features: ai github gitlab azure-devops bitbucket jira trello gitea linear youtrack redmine gerrit local-issues mcp lfs worktree submodule avatar-fetch image-view radar mermaid board review conversation history-rewrite global-search workspaces"
            ));
            assert!(!text.contains("custom build"));
        }
    }

    #[test]
    fn predicates_need_the_build_and_the_config() {
        let mut cfg = gitpoulpi_config::Config::default();
        cfg.ai.enabled = true;
        assert_eq!(ai_available(&cfg), AI);
        cfg.ai.enabled = false;
        assert!(!ai_available(&cfg));
        cfg.git.lfs_enabled = false;
        assert!(!lfs_available(&cfg));
    }

    #[cfg(not(feature = "ai"))]
    #[test]
    fn a_build_without_ai_offers_no_ai_entry_anywhere() {
        let mut cfg = gitpoulpi_config::Config::default();
        cfg.ai.enabled = true;
        assert!(!ai_available(&cfg));
        assert!(!Feature::Ai.built());
        // No palette entry, no keymap action: the tagged ones are filtered
        // and the untagged ones are not AI.
        assert!(crate::palette::PALETTE_COMMANDS
            .iter()
            .filter(|e| offered(e.requires))
            .all(|e| !e.command.is_ai()));
        assert!(crate::keymap::ACTIONS
            .iter()
            .filter(|d| d.built())
            .all(|d| !d.config_id().contains("ai_")));
        // The AI runtime object never turns on.
        let ui = crate::ai_ui::AiUi::new();
        assert!(!ui.enabled());
    }

    #[test]
    fn provider_choices_list_only_built_kinds() {
        use gitpoulpi_config::IntegrationKind;
        for kind in IntegrationKind::IMPLEMENTED {
            let built = family_built(kind.family());
            assert_eq!(PROVIDER_KINDS.contains(kind), built, "{kind:?}");
            assert_eq!(provider_kinds().contains(kind), built, "{kind:?}");
            assert_eq!(kind_built(*kind), built, "{kind:?}");
        }
        assert_eq!(
            HOSTING,
            GITHUB
                || GITLAB
                || AZURE_DEVOPS
                || BITBUCKET
                || JIRA
                || TRELLO
                || GITEA
                || LINEAR
                || YOUTRACK
                || REDMINE
                || GERRIT
                || LOCAL_ISSUES
        );
    }

    #[cfg(all(feature = "github", not(feature = "gitlab")))]
    #[test]
    fn a_github_only_build_hides_the_gitlab_choices() {
        assert!(PROVIDER_KINDS
            .iter()
            .all(|k| k.family() == gitpoulpi_config::ApiFamily::Github));
        assert!(!kind_built(gitpoulpi_config::IntegrationKind::Gitlab));
        assert!(!kind_built(
            gitpoulpi_config::IntegrationKind::GitlabSelfManaged
        ));
    }

    #[cfg(all(feature = "gitlab", not(feature = "github")))]
    #[test]
    fn a_gitlab_only_build_hides_the_github_choices() {
        assert!(PROVIDER_KINDS
            .iter()
            .all(|k| k.family() == gitpoulpi_config::ApiFamily::Gitlab));
        assert!(!kind_built(gitpoulpi_config::IntegrationKind::Github));
        assert!(!kind_built(
            gitpoulpi_config::IntegrationKind::GithubEnterprise
        ));
    }

    #[cfg(not(feature = "hosting"))]
    #[test]
    fn a_build_with_no_provider_offers_no_hosting_surface() {
        use crate::palette::{palette_matches, PaletteCommand, PaletteContext};
        assert!(PROVIDER_KINDS.is_empty());
        // No hosting keymap action is offered...
        for def in crate::keymap::ACTIONS
            .iter()
            .filter(|d| d.requires == Some(Feature::Hosting))
        {
            assert!(!def.built(), "{}", def.config_id());
        }
        // ...nor palette entry...
        let all = palette_matches(
            "",
            PaletteContext {
                ai_enabled: true,
                ..PaletteContext::default()
            },
        );
        assert!(all.iter().all(|m| !matches!(
            m.entry.command,
            PaletteCommand::Hosting(_) | PaletteCommand::StartPullRequestFromSelectedBranch
        )));
    }

    /// The palette's rows for a build, as the palette lists them.
    fn listed_commands() -> Vec<crate::palette::PaletteCommand> {
        crate::palette::palette_matches(
            "",
            crate::palette::PaletteContext {
                ai_enabled: true,
                ..crate::palette::PaletteContext::default()
            },
        )
        .into_iter()
        .map(|m| m.entry.command)
        .collect()
    }

    /// No keymap action of the feature is offered (so no key, no hint).
    fn no_action_of(feature: Feature) {
        let tagged: Vec<_> = crate::keymap::ACTIONS
            .iter()
            .filter(|d| d.requires == Some(feature))
            .collect();
        assert!(!tagged.is_empty(), "{feature:?} tags no keymap action");
        for def in tagged {
            assert!(!def.built(), "{}", def.config_id());
        }
    }

    #[test]
    fn the_carved_out_features_need_a_provider_where_they_talk_to_one() {
        assert_eq!(RADAR, cfg!(feature = "radar") && HOSTING);
        assert_eq!(BOARD, cfg!(feature = "board") && HOSTING);
        assert_eq!(REVIEW, cfg!(feature = "review") && HOSTING);
        assert_eq!(CONVERSATION, cfg!(feature = "conversation") && HOSTING);
        assert_eq!(MERMAID, cfg!(feature = "mermaid"));
    }

    #[cfg(not(all(feature = "radar", feature = "hosting")))]
    #[test]
    fn a_build_without_the_radar_offers_no_radar_surface() {
        use crate::palette::PaletteCommand;
        assert!(!Feature::Radar.built());
        no_action_of(Feature::Radar);
        assert!(listed_commands()
            .iter()
            .all(|c| !matches!(c, PaletteCommand::Radar(_))));
        // No help row for its keys either.
        let help = crate::chrome::help_body("", 600);
        assert!(!help.contains(gitpoulpi_i18n::msg::radar::HELP_OPEN.get()));
    }

    #[cfg(not(feature = "mermaid"))]
    #[test]
    fn a_build_without_mermaid_offers_no_copy_as_mermaid() {
        use crate::palette::PaletteCommand;
        assert!(!Feature::Mermaid.built());
        no_action_of(Feature::Mermaid);
        assert!(listed_commands()
            .iter()
            .all(|c| !matches!(c, PaletteCommand::CopySelectedAsMermaid)));
        let help = crate::chrome::help_body("", 600);
        assert!(!help.contains(gitpoulpi_i18n::msg::mermaid::HELP_COPY.get()));
        // The popup is inert: it never reports itself open.
        assert!(!crate::mermaid_popup::is_open());
    }

    #[cfg(not(all(feature = "board", feature = "hosting")))]
    #[test]
    fn a_build_without_the_board_offers_no_board_command() {
        use crate::palette::PaletteCommand;
        assert!(!Feature::Board.built());
        assert!(listed_commands().iter().all(|c| !matches!(
            c,
            PaletteCommand::Board(_) | PaletteCommand::Hosting(crate::HostingCommand::OpenBoard)
        )));
    }

    #[cfg(not(all(feature = "review", feature = "hosting")))]
    #[test]
    fn a_build_without_the_review_offers_no_review_command() {
        use crate::palette::PaletteCommand;
        assert!(!Feature::Review.built());
        // Not even the AI ones (`.requires(Ai)` in the catalogue, the review
        // view is what they act on).
        assert!(listed_commands().iter().all(|c| !matches!(
            c,
            PaletteCommand::Review(_)
                | PaletteCommand::Hosting(crate::HostingCommand::ReviewPullRequest)
        )));
        #[cfg(feature = "hosting")]
        assert!(!crate::review_view::supports(
            gitpoulpi_config::IntegrationKind::Github
        ));
    }

    /// Every action of the feature is tagged, and offered exactly when the
    /// build has it (the other half of [`no_action_of`], which only runs in
    /// a build without the feature).
    fn actions_follow(feature: Feature) {
        let tagged: Vec<_> = crate::keymap::ACTIONS
            .iter()
            .filter(|d| d.requires == Some(feature))
            .collect();
        assert!(!tagged.is_empty(), "{feature:?} tags no keymap action");
        for def in tagged {
            assert_eq!(def.built(), feature.built(), "{}", def.config_id());
        }
    }

    /// The History workbench: its keys (the workbench's own context, the
    /// Amend dialog's "Modify History Deeper" button, the Git-prefix chord),
    /// its palette rows, its help section, and the Amend help row follow the
    /// feature.
    #[test]
    fn the_history_workbench_surface_follows_the_feature() {
        use crate::palette::PaletteCommand;
        let on = Feature::HistoryRewrite.built();
        actions_follow(Feature::HistoryRewrite);
        // The AI key of the workbench is tagged `Ai`, and still belongs to it.
        for def in crate::keymap::ACTIONS
            .iter()
            .filter(|d| matches!(d.ctx, crate::keymap::Ctx::Rewrite))
        {
            assert_eq!(
                def.built(),
                on && offered(def.requires),
                "{}",
                def.config_id()
            );
        }
        assert_eq!(
            listed_commands()
                .iter()
                .any(|c| matches!(c, PaletteCommand::Rewrite(_))),
            on
        );
        let help = crate::chrome::help_body("", 600);
        for m in [
            gitpoulpi_i18n::msg::help_text::REWRITE_1,
            gitpoulpi_i18n::msg::help_text::REWRITE_2,
            gitpoulpi_i18n::msg::help_text::EDITING_A_COMMIT_7,
        ] {
            assert_eq!(help.contains(m.get()), on, "{}", m.get());
        }
        assert_eq!(
            help.contains(gitpoulpi_i18n::msg::help_section::REWRITE.get()),
            on,
            "the section heading goes with its rows"
        );
        // The settings that belong to the workbench alone.
        assert_eq!(
            crate::preferences::ai_action_built(gitpoulpi_config::AiAction::RewriteScript),
            on
        );
        assert!(crate::preferences::ai_action_built(
            gitpoulpi_config::AiAction::CommitMessage
        ));
    }

    /// The global search: its key, its palette row and its help row follow
    /// the feature; the title bar button is drawn only with it.
    #[test]
    fn the_global_search_surface_follows_the_feature() {
        use crate::palette::PaletteCommand;
        let on = Feature::GlobalSearch.built();
        actions_follow(Feature::GlobalSearch);
        assert_eq!(
            listed_commands()
                .iter()
                .any(|c| matches!(c, PaletteCommand::WorkspaceSearch)),
            on
        );
        let help = crate::chrome::help_body("", 600);
        assert_eq!(
            help.contains(gitpoulpi_i18n::msg::help_text::WORKSPACE_SEARCH.get()),
            on
        );
        assert_eq!(GLOBAL_SEARCH, on);
    }

    /// Workspaces: the keys (Alt+W, Alt+J, the Home page's), the palette
    /// rows, the help rows and the Workspaces tab follow the feature.
    #[test]
    fn the_workspaces_surface_follows_the_feature() {
        use crate::palette::PaletteCommand;
        let on = Feature::Workspaces.built();
        actions_follow(Feature::Workspaces);
        assert_eq!(
            listed_commands()
                .iter()
                .any(|c| matches!(c, PaletteCommand::Workspace(_))),
            on
        );
        let help = crate::chrome::help_body("", 600);
        assert_eq!(
            help.contains(gitpoulpi_i18n::msg::workspaces::HELP_SWITCH.get()),
            on
        );
        install_workspaces_open(true);
        assert_eq!(workspaces_tab_shown(), on);
        install_workspaces_open(false);
        assert!(!workspaces_tab_shown());
    }

    /// The `+` menu's Radar and Workspaces rows are independent: each is
    /// offered exactly when its own tab is not on the bar (closed, or open
    /// in the state but squeezed out of a narrow bar), whatever the other
    /// tab does.
    #[test]
    fn the_plus_menu_rows_follow_their_own_tab_only() {
        install_radar_enabled(true);
        let radar_on = offered(Some(Feature::Radar));
        for (radar_closed_, ws_open, drawn_radar, drawn_ws) in [
            (true, false, false, false),
            (false, false, true, false),
            (true, true, false, true),
            (false, true, true, true),
            // Open in the state, no room on the bar.
            (false, true, true, false),
            (false, true, false, false),
        ] {
            install_radar_closed(radar_closed_);
            install_workspaces_open(ws_open);
            install_tabs_drawn(drawn_radar, drawn_ws);
            assert_eq!(
                plus_menu_offers_radar(),
                radar_on && !drawn_radar,
                "radar closed={radar_closed_} drawn={drawn_radar}"
            );
            assert_eq!(
                plus_menu_offers_workspaces(),
                WORKSPACES && !drawn_ws,
                "workspaces open={ws_open} drawn={drawn_ws}"
            );
        }
    }

    /// Without workspaces the window is "No workspace" whatever
    /// `workspaces.toml` holds, and nothing is switched.
    #[cfg(not(feature = "workspaces"))]
    #[test]
    fn a_build_without_workspaces_sees_none_even_when_the_file_has_some() {
        let dir = tempfile::tempdir().unwrap();
        let mut all = gitpoulpi_session::Workspaces::default();
        all.workspaces.push(gitpoulpi_session::Workspace {
            id: "acme".into(),
            name: "Acme".into(),
            ..Default::default()
        });
        assert!(crate::workspace_ui::store::save_in(dir.path(), &all));
        assert!(crate::workspace_ui::store::load().workspaces.is_empty());
        assert!(crate::workspace_ui::store::active("s").is_none());
        assert!(crate::workspace_ui::store::switch("s", Some("acme")).is_none());
    }

    #[test]
    fn english_is_always_built_and_listed_first() {
        assert_eq!(languages()[0], "en");
    }
}
