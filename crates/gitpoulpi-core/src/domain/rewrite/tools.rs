//! The programs a history rewrite may need, and which rules need which.
//!
//! **The core actions need no tool at all**: identity, mailmap, anonymise,
//! dates, committer, message, trailer, content, path and blob rules run in
//! the engine with plumbing `git` only, so [`RuleKind::required_tools`] is
//! empty for them. Lua scripts run in an interpreter embedded in GitPoulpi
//! (no program to find), so a Lua script needs nothing either.
//!
//! What is left is optional and probed by the backends (`rewrite_tools`):
//! `gpg` / `ssh-keygen` / `gpgsm` to re-sign, `git-lfs` for the LFS
//! migration, `python` for external scripts (off by default in settings),
//! and a shell (`bash`, `pwsh`, `sh`) for shell scripts.
//!
//! Everything here is pure; the probing lives in `gitpoulpi-cli-backend`.

use serde::{Deserialize, Serialize};

use super::recipe::{Recipe, RuleKind, ScriptLanguage, SignaturePolicy};

/// An external program a rewrite can use. Lua is deliberately absent: it is
/// embedded.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Tool {
    Gpg,
    SshKeygen,
    Gpgsm,
    GitLfs,
    /// External scripts. Off by default in settings.
    Python,
    /// An installed Lua for external Lua scripts (the embedded Lua needs
    /// nothing). Off by default in settings.
    Lua,
    Bash,
    Pwsh,
    Sh,
}

impl Tool {
    /// Every tool, in the order the workbench lists them.
    pub const ALL: [Tool; 9] = [
        Tool::Gpg,
        Tool::SshKeygen,
        Tool::Gpgsm,
        Tool::GitLfs,
        Tool::Python,
        Tool::Lua,
        Tool::Bash,
        Tool::Pwsh,
        Tool::Sh,
    ];

    /// The name users know it by (`git-lfs`, `ssh-keygen`...).
    pub fn name(self) -> &'static str {
        match self {
            Tool::Gpg => "gpg",
            Tool::SshKeygen => "ssh-keygen",
            Tool::Gpgsm => "gpgsm",
            Tool::GitLfs => "git-lfs",
            Tool::Python => "python",
            Tool::Lua => "lua",
            Tool::Bash => "bash",
            Tool::Pwsh => "pwsh",
            Tool::Sh => "sh",
        }
    }

    /// The suffix of the test seam `GITPOULPI_TOOL_<NAME>`.
    pub fn env_suffix(self) -> &'static str {
        match self {
            Tool::Gpg => "GPG",
            Tool::SshKeygen => "SSH_KEYGEN",
            Tool::Gpgsm => "GPGSM",
            Tool::GitLfs => "GIT_LFS",
            Tool::Python => "PYTHON",
            Tool::Lua => "LUA",
            Tool::Bash => "BASH",
            Tool::Pwsh => "PWSH",
            Tool::Sh => "SH",
        }
    }

    /// The oldest version that works, as `(major, minor)`.
    pub fn min_version(self) -> Option<(u32, u32)> {
        match self {
            Tool::GitLfs => Some((3, 0)),
            Tool::Python => Some((3, 8)),
            _ => None,
        }
    }

    /// The features that use this tool.
    pub fn needed_for(self) -> Vec<RewriteFeature> {
        match self {
            Tool::Gpg | Tool::SshKeygen | Tool::Gpgsm => vec![RewriteFeature::ReSign],
            Tool::GitLfs => vec![RewriteFeature::LfsMigration],
            Tool::Python | Tool::Lua => vec![RewriteFeature::ExternalScripts],
            Tool::Bash | Tool::Pwsh | Tool::Sh => vec![RewriteFeature::ShellScripts],
        }
    }

    /// The i18n key (in `[rewrite_tools]`) of the install hint on `os`.
    pub fn hint_key(self, os: OsFamily) -> &'static str {
        use OsFamily::{Linux, MacOs, Windows};
        match (self, os) {
            (Tool::Gpg, Windows) => "tool_hint_gpg_windows",
            (Tool::Gpg, Linux) => "tool_hint_gpg_linux",
            (Tool::Gpg, MacOs) => "tool_hint_gpg_macos",
            (Tool::SshKeygen, Windows) => "tool_hint_ssh_keygen_windows",
            (Tool::SshKeygen, Linux) => "tool_hint_ssh_keygen_linux",
            (Tool::SshKeygen, MacOs) => "tool_hint_ssh_keygen_macos",
            (Tool::Gpgsm, Windows) => "tool_hint_gpgsm_windows",
            (Tool::Gpgsm, Linux) => "tool_hint_gpgsm_linux",
            (Tool::Gpgsm, MacOs) => "tool_hint_gpgsm_macos",
            (Tool::GitLfs, Windows) => "tool_hint_git_lfs_windows",
            (Tool::GitLfs, Linux) => "tool_hint_git_lfs_linux",
            (Tool::GitLfs, MacOs) => "tool_hint_git_lfs_macos",
            (Tool::Python, Windows) => "tool_hint_python_windows",
            (Tool::Python, Linux) => "tool_hint_python_linux",
            (Tool::Python, MacOs) => "tool_hint_python_macos",
            (Tool::Lua, Windows) => "tool_hint_lua_windows",
            (Tool::Lua, Linux) => "tool_hint_lua_linux",
            (Tool::Lua, MacOs) => "tool_hint_lua_macos",
            (Tool::Bash, Windows) => "tool_hint_bash_windows",
            (Tool::Bash, Linux) => "tool_hint_bash_linux",
            (Tool::Bash, MacOs) => "tool_hint_bash_macos",
            (Tool::Pwsh, Windows) => "tool_hint_pwsh_windows",
            (Tool::Pwsh, Linux) => "tool_hint_pwsh_linux",
            (Tool::Pwsh, MacOs) => "tool_hint_pwsh_macos",
            (Tool::Sh, Windows) => "tool_hint_sh_windows",
            (Tool::Sh, Linux) => "tool_hint_sh_linux",
            (Tool::Sh, MacOs) => "tool_hint_sh_macos",
        }
    }
}

/// The OS family an install hint is written for.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OsFamily {
    Windows,
    Linux,
    MacOs,
}

impl OsFamily {
    /// The family this build runs on (other Unixes read as Linux).
    pub const fn current() -> Self {
        if cfg!(windows) {
            OsFamily::Windows
        } else if cfg!(target_os = "macos") {
            OsFamily::MacOs
        } else {
            OsFamily::Linux
        }
    }
}

/// A rewrite capability that depends on tools.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RewriteFeature {
    /// Re-signing rewritten commits (S19).
    ReSign,
    /// Moving blobs into or out of LFS (S21).
    LfsMigration,
    /// Scripts run by an external command (S11).
    ExternalScripts,
    /// Shell scripts (S11).
    ShellScripts,
}

/// The `gpg.format` of a repository: what signs a commit.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SigningFormat {
    #[default]
    OpenPgp,
    Ssh,
    X509,
}

impl SigningFormat {
    /// `gpg.format` value to format; unknown values read as OpenPGP, which
    /// is what git does.
    pub fn from_config(value: Option<&str>) -> Self {
        match value.map(|v| v.trim().to_ascii_lowercase()).as_deref() {
            Some("ssh") => Self::Ssh,
            Some("x509") => Self::X509,
            _ => Self::OpenPgp,
        }
    }

    /// The program that signs for this format.
    pub fn tool(self) -> Tool {
        match self {
            Self::OpenPgp => Tool::Gpg,
            Self::Ssh => Tool::SshKeygen,
            Self::X509 => Tool::Gpgsm,
        }
    }
}

/// What a rule or option asks of the machine.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ToolRequirement {
    /// This tool.
    Tool(Tool),
    /// Any one of these (a shell script runs under whichever shell exists).
    AnyOf(&'static [Tool]),
}

/// The shells a shell script may run under.
pub const SHELLS: &[Tool] = &[Tool::Bash, Tool::Pwsh, Tool::Sh];

impl ToolRequirement {
    /// Whether `tools` satisfies this requirement.
    pub fn met_by(&self, tools: &RewriteTools) -> bool {
        match self {
            Self::Tool(t) => tools.is_usable(*t),
            Self::AnyOf(any) => any.iter().any(|t| tools.is_usable(*t)),
        }
    }

    /// The tools to offer when this requirement is not met.
    pub fn candidates(&self) -> Vec<Tool> {
        match self {
            Self::Tool(t) => vec![*t],
            Self::AnyOf(any) => any.to_vec(),
        }
    }
}

impl RuleKind {
    /// The tools this rule needs. **Empty for every metadata, content and
    /// path rule**: only the LFS migration and external or shell scripts
    /// need a program. An embedded Lua script needs none.
    pub fn required_tools(&self) -> Vec<ToolRequirement> {
        match self {
            Self::LfsImport(_) | Self::LfsExport(_) => vec![ToolRequirement::Tool(Tool::GitLfs)],
            Self::Script(s) => match s.language {
                ScriptLanguage::Lua => Vec::new(),
                ScriptLanguage::Python => vec![ToolRequirement::Tool(Tool::Python)],
                ScriptLanguage::LuaExternal => vec![ToolRequirement::Tool(Tool::Lua)],
                ScriptLanguage::Shell => vec![ToolRequirement::AnyOf(SHELLS)],
            },
            _ => Vec::new(),
        }
    }
}

impl Recipe {
    /// The tools the enabled rules and the signature choice need. `format`
    /// is the repository's `gpg.format`, which decides what re-signing runs.
    /// Empty for a recipe of core actions only.
    pub fn required_tools(&self, format: SigningFormat) -> Vec<ToolRequirement> {
        let mut out: Vec<ToolRequirement> = Vec::new();
        let mut push = |r: ToolRequirement| {
            if !out.contains(&r) {
                out.push(r);
            }
        };
        for (_, rule) in self.enabled_rules() {
            for r in rule.kind.required_tools() {
                push(r);
            }
        }
        if matches!(
            self.options.signatures,
            Some(SignaturePolicy::ResignSigned | SignaturePolicy::ResignAll)
        ) {
            push(ToolRequirement::Tool(format.tool()));
        }
        out
    }
}

/// What the probe found for one tool.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "status", rename_all = "snake_case")]
pub enum ToolStatus {
    Installed {
        /// `""` when the program has no version output (`ssh-keygen`, `sh`).
        version: String,
    },
    TooOld {
        version: String,
        /// `major.minor`.
        min: String,
    },
    Missing,
    /// The program is there but something it needs is not set (an SSH or
    /// X.509 signing key).
    NotConfigured {
        reason: String,
    },
}

impl ToolStatus {
    pub fn is_usable(&self) -> bool {
        matches!(self, ToolStatus::Installed { .. })
    }

    /// Classifies `version` (digits and dots) against `tool`'s minimum.
    pub fn from_version(tool: Tool, version: &str) -> Self {
        let version = version.to_string();
        let Some((min_major, min_minor)) = tool.min_version() else {
            return ToolStatus::Installed { version };
        };
        let mut nums = version.split('.').map(|n| n.parse::<u32>().unwrap_or(0));
        let have = (nums.next().unwrap_or(0), nums.next().unwrap_or(0));
        if have >= (min_major, min_minor) {
            ToolStatus::Installed { version }
        } else {
            ToolStatus::TooOld {
                version,
                min: format!("{min_major}.{min_minor}"),
            }
        }
    }
}

/// One row of [`RewriteTools`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ToolCheck {
    pub tool: Tool,
    pub needed_for: Vec<RewriteFeature>,
    pub status: ToolStatus,
    /// The i18n key of the install hint for this OS (`[rewrite_tools]`).
    pub hint_key: String,
}

impl ToolCheck {
    pub fn new(tool: Tool, status: ToolStatus, os: OsFamily) -> Self {
        Self {
            tool,
            needed_for: tool.needed_for(),
            status,
            hint_key: tool.hint_key(os).to_string(),
        }
    }
}

/// The result of `rewrite_check_signing`: one dummy payload signed and
/// verified with the repository's own signing setup. `reason` is `None`
/// when it worked; else `tool_missing` or `signing_failed` with the
/// program's own words in `detail`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SigningCheck {
    pub format: SigningFormat,
    pub tool: Tool,
    /// The i18n key of the install hint for this OS (`[rewrite_tools]`).
    pub hint_key: String,
    /// `None`: signing works.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reason: Option<super::recipe::RewriteRefusal>,
    #[serde(default)]
    pub detail: String,
    /// How long one signature took, in milliseconds (0 when it failed).
    #[serde(default)]
    pub millis: u64,
}

impl SigningCheck {
    pub fn is_ok(&self) -> bool {
        self.reason.is_none()
    }

    /// The time `commits` signatures take, in milliseconds.
    pub fn estimate_millis(&self, commits: u64) -> u64 {
        self.millis.saturating_mul(commits)
    }
}

/// The result of `rewrite_tools`: one check per [`Tool`]. Lua is not listed:
/// it is embedded and always available.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct RewriteTools {
    pub tools: Vec<ToolCheck>,
    /// Which external script languages `rewrite.external_scripts` allows. The
    /// backend leaves it `None`; the daemon fills it from the settings.
    /// `None` is "not known": nothing is greyed on a guess.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub external_scripts: Option<ScriptSwitches>,
    /// What signs commits in this repository (`gpg.format`): the tool a
    /// re-signing run needs. Defaults to OpenPGP, as git does.
    #[serde(default)]
    pub signing_format: SigningFormat,
}

/// The languages the `rewrite.external_scripts` setting switches on.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct ScriptSwitches {
    pub shell: bool,
    pub python: bool,
    pub lua: bool,
}

impl ScriptSwitches {
    /// Whether the setting allows `language` (the embedded Lua always).
    pub fn allows(&self, language: ScriptLanguage) -> bool {
        match language {
            ScriptLanguage::Lua => true,
            ScriptLanguage::Shell => self.shell,
            ScriptLanguage::Python => self.python,
            ScriptLanguage::LuaExternal => self.lua,
        }
    }
}

impl RewriteTools {
    pub fn get(&self, tool: Tool) -> Option<&ToolCheck> {
        self.tools.iter().find(|c| c.tool == tool)
    }

    pub fn is_usable(&self, tool: Tool) -> bool {
        self.get(tool).is_some_and(|c| c.status.is_usable())
    }

    /// The requirements of `needs` that are not met, for Preview's
    /// `ToolMissing` refusal and the greyed rule kinds.
    pub fn unmet(&self, needs: &[ToolRequirement]) -> Vec<ToolRequirement> {
        needs.iter().filter(|r| !r.met_by(self)).copied().collect()
    }
}

#[cfg(test)]
mod tests {
    use super::super::recipe::{
        LfsImportRule, Rule, RuleKind, ScriptRef, ScriptSource, TrailerRule,
    };
    use super::super::{Recipe, Scope};
    use super::*;
    use std::collections::BTreeMap;

    fn script(language: ScriptLanguage) -> RuleKind {
        RuleKind::Script(ScriptRef {
            language,
            source: ScriptSource::Inline { text: "x".into() },
            params: BTreeMap::new(),
        })
    }

    #[test]
    fn metadata_and_content_rules_need_no_tool() {
        let kinds = [
            RuleKind::Mailmap {
                apply_to: Default::default(),
                file: None,
            },
            RuleKind::CommitterFromAuthor,
            RuleKind::DateShift {
                by_seconds: 60,
                apply_to: Default::default(),
            },
            RuleKind::RemovePaths {
                globs: vec!["a".into()],
            },
            RuleKind::RemoveBlobsOver { bytes: 10 },
            RuleKind::Trailer(TrailerRule::add("K", "v")),
            script(ScriptLanguage::Lua),
        ];
        for kind in kinds {
            assert!(kind.required_tools().is_empty(), "{}", kind.name());
        }
    }

    #[test]
    fn lfs_external_and_shell_name_their_tools() {
        let lfs = RuleKind::LfsImport(LfsImportRule {
            patterns: vec!["*.psd".into()],
            min_size: None,
        });
        assert_eq!(
            lfs.required_tools(),
            vec![ToolRequirement::Tool(Tool::GitLfs)]
        );
        assert_eq!(
            script(ScriptLanguage::Python).required_tools(),
            vec![ToolRequirement::Tool(Tool::Python)]
        );
        assert_eq!(
            script(ScriptLanguage::LuaExternal).required_tools(),
            vec![ToolRequirement::Tool(Tool::Lua)]
        );
        assert!(script(ScriptLanguage::Lua).required_tools().is_empty());
        assert_eq!(
            script(ScriptLanguage::Shell).required_tools(),
            vec![ToolRequirement::AnyOf(SHELLS)]
        );
    }

    #[test]
    fn a_recipe_needs_the_signing_tool_only_when_it_re_signs() {
        let mut recipe = Recipe::new(
            "t",
            Scope::default(),
            vec![Rule::new(RuleKind::CommitterFromAuthor)],
        );
        assert!(recipe.required_tools(SigningFormat::Ssh).is_empty());
        recipe.options.signatures = Some(SignaturePolicy::Strip);
        assert!(recipe.required_tools(SigningFormat::Ssh).is_empty());
        recipe.options.signatures = Some(SignaturePolicy::ResignAll);
        assert_eq!(
            recipe.required_tools(SigningFormat::Ssh),
            vec![ToolRequirement::Tool(Tool::SshKeygen)]
        );
        assert_eq!(
            recipe.required_tools(SigningFormat::X509),
            vec![ToolRequirement::Tool(Tool::Gpgsm)]
        );
        assert_eq!(
            recipe.required_tools(SigningFormat::from_config(None)),
            vec![ToolRequirement::Tool(Tool::Gpg)]
        );
    }

    #[test]
    fn a_disabled_rule_needs_nothing() {
        let mut rule = Rule::new(script(ScriptLanguage::Shell));
        rule.enabled = false;
        let recipe = Recipe::new("t", Scope::default(), vec![rule]);
        assert!(recipe.required_tools(SigningFormat::OpenPgp).is_empty());
    }

    #[test]
    fn versions_are_classified_against_the_minimum() {
        assert!(ToolStatus::from_version(Tool::GitLfs, "3.5.1").is_usable());
        assert!(matches!(
            ToolStatus::from_version(Tool::GitLfs, "2.13.3"),
            ToolStatus::TooOld { .. }
        ));
        assert!(ToolStatus::from_version(Tool::Gpg, "1.0").is_usable());
    }

    #[test]
    fn any_of_is_met_by_one_usable_shell() {
        let tools = RewriteTools {
            external_scripts: None,
            signing_format: SigningFormat::default(),
            tools: vec![
                ToolCheck::new(Tool::Bash, ToolStatus::Missing, OsFamily::Linux),
                ToolCheck::new(
                    Tool::Sh,
                    ToolStatus::Installed { version: "".into() },
                    OsFamily::Linux,
                ),
            ],
        };
        let need = ToolRequirement::AnyOf(SHELLS);
        assert!(need.met_by(&tools));
        assert!(tools.unmet(&[need]).is_empty());
        assert_eq!(tools.unmet(&[ToolRequirement::Tool(Tool::Gpg)]).len(), 1);
    }

    #[test]
    fn every_tool_has_a_hint_key_per_os_and_a_feature() {
        for tool in Tool::ALL {
            assert!(!tool.needed_for().is_empty());
            for os in [OsFamily::Windows, OsFamily::Linux, OsFamily::MacOs] {
                assert!(tool.hint_key(os).starts_with("tool_hint_"));
            }
        }
    }
}
