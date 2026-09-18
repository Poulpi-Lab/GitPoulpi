//! What the workbench knows about the external tools a rule or an option
//! needs (the history-rewrite design, "Tools"): asked of the
//! daemon when the workbench opens, never on the render thread, and turned
//! into the reason and install hint a greyed-out rule shows.

use gitpoulpi_core::domain::rewrite::{
    OsFamily, Recipe, RewriteTools, Rule, SigningFormat, Tool, ToolStatus,
};
use gitpoulpi_i18n::{msg::rewrite_tools as hint, M};

use super::model::rule_tools_unmet;

/// Where the answer of the tool check stands.
#[derive(Default)]
pub(crate) enum ToolsState {
    /// Not asked yet.
    #[default]
    Unknown,
    /// The last answer.
    Known(RewriteTools),
}

impl ToolsState {
    pub(crate) fn tools(&self) -> Option<&RewriteTools> {
        match self {
            ToolsState::Known(t) => Some(t),
            _ => None,
        }
    }
}

/// The install hint of `tool` on `os`.
pub(crate) fn tool_hint(tool: Tool, os: OsFamily) -> M {
    use OsFamily::{Linux, MacOs, Windows};
    match (tool, os) {
        (Tool::Gpg, Windows) => hint::TOOL_HINT_GPG_WINDOWS,
        (Tool::Gpg, Linux) => hint::TOOL_HINT_GPG_LINUX,
        (Tool::Gpg, MacOs) => hint::TOOL_HINT_GPG_MACOS,
        (Tool::SshKeygen, Windows) => hint::TOOL_HINT_SSH_KEYGEN_WINDOWS,
        (Tool::SshKeygen, Linux) => hint::TOOL_HINT_SSH_KEYGEN_LINUX,
        (Tool::SshKeygen, MacOs) => hint::TOOL_HINT_SSH_KEYGEN_MACOS,
        (Tool::Gpgsm, Windows) => hint::TOOL_HINT_GPGSM_WINDOWS,
        (Tool::Gpgsm, Linux) => hint::TOOL_HINT_GPGSM_LINUX,
        (Tool::Gpgsm, MacOs) => hint::TOOL_HINT_GPGSM_MACOS,
        (Tool::GitLfs, Windows) => hint::TOOL_HINT_GIT_LFS_WINDOWS,
        (Tool::GitLfs, Linux) => hint::TOOL_HINT_GIT_LFS_LINUX,
        (Tool::GitLfs, MacOs) => hint::TOOL_HINT_GIT_LFS_MACOS,
        (Tool::Python, Windows) => hint::TOOL_HINT_PYTHON_WINDOWS,
        (Tool::Python, Linux) => hint::TOOL_HINT_PYTHON_LINUX,
        (Tool::Python, MacOs) => hint::TOOL_HINT_PYTHON_MACOS,
        (Tool::Lua, Windows) => hint::TOOL_HINT_LUA_WINDOWS,
        (Tool::Lua, Linux) => hint::TOOL_HINT_LUA_LINUX,
        (Tool::Lua, MacOs) => hint::TOOL_HINT_LUA_MACOS,
        (Tool::Bash, Windows) => hint::TOOL_HINT_BASH_WINDOWS,
        (Tool::Bash, Linux) => hint::TOOL_HINT_BASH_LINUX,
        (Tool::Bash, MacOs) => hint::TOOL_HINT_BASH_MACOS,
        (Tool::Pwsh, Windows) => hint::TOOL_HINT_PWSH_WINDOWS,
        (Tool::Pwsh, Linux) => hint::TOOL_HINT_PWSH_LINUX,
        (Tool::Pwsh, MacOs) => hint::TOOL_HINT_PWSH_MACOS,
        (Tool::Sh, Windows) => hint::TOOL_HINT_SH_WINDOWS,
        (Tool::Sh, Linux) => hint::TOOL_HINT_SH_LINUX,
        (Tool::Sh, MacOs) => hint::TOOL_HINT_SH_MACOS,
    }
}

/// Why a rule is greyed out: the missing tools with their status and hint.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct MissingTool {
    pub tool: Tool,
    /// What the check found (`Missing`, `TooOld`, `NotConfigured`).
    pub status: ToolStatus,
    pub hint: M,
}

/// The tools `rule` needs that the machine lacks. Empty while the check has
/// not answered (nothing is greyed on a guess).
pub(crate) fn missing_for_rule(state: &ToolsState, rule: &Rule) -> Vec<MissingTool> {
    let Some(tools) = state.tools() else {
        return Vec::new();
    };
    describe(tools, rule_tools_unmet(rule, tools))
}

/// The tools the whole recipe needs (its rules and its signature choice)
/// that the machine lacks.
pub(crate) fn missing_for_recipe(
    state: &ToolsState,
    recipe: &Recipe,
    format: SigningFormat,
) -> Vec<MissingTool> {
    let Some(tools) = state.tools() else {
        return Vec::new();
    };
    let unmet = tools.unmet(&recipe.required_tools(format));
    describe(tools, unmet.iter().flat_map(|r| r.candidates()).collect())
}

fn describe(tools: &RewriteTools, wanted: Vec<Tool>) -> Vec<MissingTool> {
    let mut out: Vec<MissingTool> = Vec::new();
    for tool in wanted {
        if out.iter().any(|m| m.tool == tool) {
            continue;
        }
        let status = tools
            .get(tool)
            .map(|c| c.status.clone())
            .unwrap_or(ToolStatus::Missing);
        out.push(MissingTool {
            tool,
            status,
            hint: tool_hint(tool, OsFamily::current()),
        });
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use gitpoulpi_core::domain::rewrite::{LfsImportRule, RuleKind, ToolCheck};

    fn lfs_rule() -> Rule {
        Rule::new(RuleKind::LfsImport(LfsImportRule {
            patterns: vec!["*.bin".into()],
            min_size: None,
        }))
    }

    fn known(status: ToolStatus) -> ToolsState {
        ToolsState::Known(RewriteTools {
            external_scripts: None,
            signing_format: Default::default(),
            tools: vec![ToolCheck::new(Tool::GitLfs, status, OsFamily::current())],
        })
    }

    #[test]
    fn nothing_is_greyed_before_the_check_answers() {
        assert!(missing_for_rule(&ToolsState::Unknown, &lfs_rule()).is_empty());
    }

    #[test]
    fn a_missing_tool_greys_its_rule_with_the_status_and_a_hint() {
        let state = known(ToolStatus::Missing);
        let missing = missing_for_rule(&state, &lfs_rule());
        assert_eq!(missing.len(), 1);
        assert_eq!(missing[0].tool, Tool::GitLfs);
        assert_eq!(missing[0].status, ToolStatus::Missing);
        assert!(!missing[0].hint.get().is_empty());
        let ok = known(ToolStatus::Installed {
            version: "3.4".into(),
        });
        assert!(missing_for_rule(&ok, &lfs_rule()).is_empty());
    }

    #[test]
    fn a_rule_that_needs_no_tool_is_never_greyed() {
        let state = known(ToolStatus::Missing);
        let plain = Rule::new(RuleKind::CommitterFromAuthor);
        assert!(missing_for_rule(&state, &plain).is_empty());
    }
}
