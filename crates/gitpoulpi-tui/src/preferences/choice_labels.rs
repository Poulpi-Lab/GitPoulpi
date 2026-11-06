//! The words the Settings comboboxes show for a choice. The stored value is
//! always the raw schema word (`ask`, `row_tint`); only the display is
//! translated, so a language switch never touches `config.toml`.

use gitpoulpi_i18n::msg::settings_value as v;
use gitpoulpi_i18n::M;

/// The display text of `raw` for the row writing config key `key`: a
/// per-key word where the same raw value reads differently (`off` as
/// `Off`), then the shared word, else `raw` itself (encodings, terminal
/// presets, profile names).
pub(super) fn label(key: &str, raw: &str) -> String {
    keyed(key, raw)
        .or_else(|| shared(key, raw))
        .map_or_else(|| raw.to_string(), |m| m.get().to_string())
}

/// The group headings of the Add preset combobox (`detected`, `explorer`,
/// `multiplexer`, `terminal`, `editor`).
pub(super) fn open_in_group(group: &str) -> String {
    use gitpoulpi_i18n::msg::settings_open_in as t;
    match group {
        "detected" => t::GROUP_DETECTED,
        "explorer" | "filemanager" => t::GROUP_EXPLORER,
        "multiplexer" => t::GROUP_MULTIPLEXER,
        "terminal" => t::GROUP_TERMINAL,
        "git" => t::GROUP_GIT,
        "agent" => t::GROUP_AGENT,
        "web" => t::GROUP_WEB,
        _ => t::GROUP_EDITOR,
    }
    .get()
    .to_string()
}

fn keyed(key: &str, raw: &str) -> Option<M> {
    use gitpoulpi_i18n::msg::activity_log as log;
    Some(match (key, raw) {
        ("log.level", "error") => log::LEVEL_ERROR,
        ("log.level", "warn") => log::LEVEL_WARN,
        ("log.level", "info") => log::LEVEL_INFO,
        ("log.level", "debug") => log::LEVEL_DEBUG,
        ("log.level", "trace") => log::LEVEL_TRACE,
        ("log.commit_count", "system-git") => v::COUNT_SYSTEM_GIT,
        ("log.commit_count", "libgit2") => v::COUNT_LIBGIT2,
        (
            "general.startup_tab_in_repo"
            | "general.startup_tab_outside_repo"
            | "general.startup_tab_launcher",
            raw,
        ) => match raw {
            "current-repo" => v::STARTUP_CURRENT_REPO,
            "last" => v::STARTUP_LAST,
            "home" => v::STARTUP_HOME,
            "radar" => v::STARTUP_RADAR,
            _ => return None,
        },
        ("updates.mode", "auto") => gitpoulpi_i18n::msg::updates::MODE_AUTO,
        ("updates.mode", "notify") => gitpoulpi_i18n::msg::updates::MODE_NOTIFY,
        ("updates.mode", "off") => gitpoulpi_i18n::msg::updates::MODE_OFF,
        ("updates.channel", "stable") => gitpoulpi_i18n::msg::updates::CHANNEL_STABLE,
        ("updates.channel", "prerelease") => gitpoulpi_i18n::msg::updates::CHANNEL_PRERELEASE,
        ("mcp.tls", "off") => gitpoulpi_i18n::msg::mcp_security::TLS_OFF,
        ("mcp.tls", "self_signed") => gitpoulpi_i18n::msg::mcp_security::TLS_SELF_SIGNED,
        ("mcp.tls", "files") => gitpoulpi_i18n::msg::mcp_security::TLS_FILES,
        ("ui.graph_lane_background", "off") => v::LANE_OFF,
        ("ui.graph_lane_background", "row_tint") => v::ROW_TINT,
        ("ui.graph_row_tint_strength", "strong") => v::STRONG,
        ("ui.graph_row_tint_strength", "medium") => v::MEDIUM,
        ("ui.graph_row_tint_strength", "subtle") => v::SUBTLE,
        ("ui.toolbar_position", "top") | ("ui.statusbar_position", "top") => v::POS_TOP,
        ("ui.toolbar_position", "bottom") | ("ui.statusbar_position", "bottom") => v::POS_BOTTOM,
        ("ui.tabs_position", "top") => v::POS_TOP,
        ("ui.tabs_position", "bottom") => v::POS_BOTTOM,
        ("ui.tabs_position", "left") | ("ui.toolbar_position", "left") => v::POS_LEFT,
        ("ui.toolbar_position", "right") => v::POS_RIGHT,
        ("ui.toolbar_align", "start") => v::ALIGN_START,
        ("ui.toolbar_align", "center") => v::ALIGN_CENTER,
        ("ui.toolbar_align", "end") => v::ALIGN_END,
        ("editor.eol", "auto") => v::EOL_AUTO,
        ("editor.eol", "lf") => v::EOL_LF,
        ("editor.eol", "crlf") => v::EOL_CRLF,
        ("ui.pr_terminology", "auto") => v::PR_AUTO,
        ("ui.pr_terminology", "pull_request") => v::PULL_REQUEST,
        ("ui.pr_terminology", "merge_request") => v::MERGE_REQUEST,
        ("ui.input_color", "theme") => v::INPUT_THEME,
        ("ui.input_color", "gray" | "grey") => v::INPUT_GRAY,
        ("ui.input_color", "blue") => v::INPUT_BLUE,
        ("ui.input_color", "slate") => v::INPUT_SLATE,
        ("ui.input_color", "dark") => v::INPUT_DARK,
        ("ui.input_color", "custom") => v::INPUT_CUSTOM,
        ("ui.sidebar_sections_scope", "repository") => v::SECTIONS_REPOSITORY,
        ("ui.sidebar_sections_scope", "global") => v::SECTIONS_GLOBAL,
        ("ui.column_resize", "line") => v::RESIZE_LINE,
        ("ui.column_resize", "header") => v::RESIZE_HEADER,
        ("ui.log_columns_layout", "fit") => v::LAYOUT_FIT,
        ("ui.log_columns_layout", "scroll") => v::LAYOUT_SCROLL,
        ("ui.blame_side", "right") => v::BLAME_RIGHT,
        ("ui.blame_side", "left") => v::BLAME_LEFT,
        ("ui.image_view_text_fallback", "halfblock") => {
            gitpoulpi_i18n::msg::settings_images::FALLBACK_HALFBLOCK
        }
        ("ui.image_view_text_fallback", "card") => {
            gitpoulpi_i18n::msg::settings_images::FALLBACK_CARD
        }
        _ => return None,
    })
}

fn shared(key: &str, raw: &str) -> Option<M> {
    // Rows whose options are data, not schema words.
    if matches!(
        key,
        "general.active_profile"
            | "ui.language"
            | "git.default_encoding"
            | "ssh.key_type"
            | "general.default_backend"
            | "editor.tab_size"
            | "ui.graph_color_theme"
            | "ui.launch"
            | "ui.terminal_launch"
            | "ui.open_in_default"
            | "ui.image_editor_default"
    ) {
        return None;
    }
    Some(match raw {
        "off" => v::OFF,
        "none" => v::NONE,
        "ask" => v::ASK,
        "always" => v::ALWAYS,
        "never" => v::NEVER,
        "direct" => v::DIRECT,
        "merge" => v::MERGE,
        "rebase" => v::REBASE,
        "dark" => v::DARK,
        "light" => v::LIGHT,
        "system" => v::SYSTEM,
        "lane" => v::LANE,
        "author" => v::AUTHOR,
        "branch" => v::BRANCH,
        "inline" => v::INLINE,
        "split" => v::SPLIT,
        "graph" => v::GRAPH,
        "initials" => v::INITIALS,
        "expanded" => v::EXPANDED,
        "collapsed" => v::COLLAPSED,
        "buttons" => v::BUTTONS,
        "image" => v::IMAGE,
        "text" => v::TEXT,
        _ => return None,
    })
}
