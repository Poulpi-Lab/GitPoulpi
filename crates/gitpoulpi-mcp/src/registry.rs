//! Generates the MCP tool list by walking `gitpoulpi_commands::cli::Cli`'s
//! clap tree -- the same tree the CLI parses -- rather than hand-maintaining
//! a second list that could drift from it. A CLI leaf that gains, loses, or
//! renames an argument changes its tool's schema the next time this runs,
//! with nothing here to update by hand.
//!
//! A "leaf" is any subcommand with no further subcommands of its own
//! (`status`, `branch create`, `tag delete-remote`, ...); a node that only
//! dispatches to more subcommands (`branch`, `tag`, ...) is never a tool.

use clap::{Command, CommandFactory};
use gitpoulpi_commands::cli::Cli;
use serde_json::Value;

use crate::{policy, schema};

/// One MCP tool: everything `server.rs` needs to advertise it
/// (`list_tools`) and to run it (`call_tool`) by turning its arguments
/// back into the exact argv the `gitpoulpi` binary would have parsed.
#[derive(Debug, Clone)]
pub struct ToolSpec {
    /// `snake_case`, unique: `branch_create`, `conflicts_resolve`.
    pub name: String,
    /// The subcommand path from the root, as clap spells it
    /// (`["branch", "create"]`, `["tag", "delete-remote"]`).
    pub cli_path: Vec<String>,
    pub description: String,
    pub input_schema: Value,
    pub read_only: bool,
    pub destructive: bool,
    pub idempotent: bool,
    /// The leaf's built `clap::Command`, kept so `argv.rs` can read the
    /// exact same argument metadata `schema.rs` built the schema from,
    /// instead of a second, possibly-drifting description of the same
    /// arguments.
    pub(crate) command: Command,
    /// Arguments of the CLI leaf this tool does not offer (not in the schema,
    /// refused if sent): the operator's gates, such as `allow_sensitive`
    /// while `[mcp] allow_sensitive_staging` is off.
    pub(crate) withheld: Vec<&'static str>,
}

/// CLI leaves deliberately not exposed as MCP tools, each with the reason -
/// checked by `excluded_entries_name_real_cli_leaves` against a fresh
/// clap tree, so a renamed or removed subcommand here is caught rather
/// than silently going stale.
pub const EXCLUDED: &[(&[&str], &str)] = &[
    (
        &["hosting", "integrations", "trust"],
        "accepting a changed address, account or certificate is the user's decision; an agent is told the security check needs the user",
    ),
    (
        &["ai", "trust"],
        "binding an API key to another address is the user's decision",
    ),
    (
        &["mcp", "serve"],
        "the MCP server cannot spawn/serve a copy of itself",
    ),
    (
        &["mcp", "token"],
        "prints, copies and rotates the key that guards this very server: an agent must not read or replace it",
    ),
    (
        &["completions"],
        "shell completion generation, not a GitBackend operation",
    ),
    (
        &["shell-init"],
        "prints a wrapper function for the user's own shell, not a GitBackend operation",
    ),
    (
        &["desktop", "shortcut", "status"],
        "launcher shortcuts are the user's desktop, not a repository; they need the TUI's icon renderer",
    ),
    (
        &["desktop", "shortcut", "install"],
        "writes files into the user's launcher, Start menu and Desktop",
    ),
    (
        &["desktop", "shortcut", "remove"],
        "deletes files from the user's launcher, Start menu and Desktop",
    ),
    (
        &["desktop", "folder-menu", "status"],
        "the file manager menu is the user's desktop, not a repository",
    ),
    (
        &["desktop", "folder-menu", "install"],
        "writes entries into the user's file manager menus and registry",
    ),
    (
        &["desktop", "folder-menu", "remove"],
        "deletes entries from the user's file manager menus and registry",
    ),
    (
        &["alias", "create"],
        "writes a command into the user's PATH; an agent never changes the user's shell",
    ),
    (
        &["alias", "remove"],
        "deletes a command from the user's PATH; an agent never changes the user's shell",
    ),
    (
        &["installation", "show"],
        "describes this machine's GitPoulpi installations (folders, environments); local, like desktop and alias",
    ),
    (
        &["installation", "set"],
        "changes this installation's own names, colour, aliases and environment in the user's config",
    ),
    (
        &["path", "add"],
        "edits the user's shell rc files or Windows user PATH",
    ),
    (
        &["path", "remove"],
        "edits the user's shell rc files or Windows user PATH",
    ),
    (
        &["uninstall"],
        "removes GitPoulpi's own program files, aliases, PATH entries and shortcuts from the machine",
    ),
    (
        &["launch"],
        "starts a terminal window on the user's desktop; the launcher shortcuts' own entry point",
    ),
    (
        &["daemon", "run"],
        "daemon process lifecycle, not a GitBackend operation",
    ),
    (
        &["daemon", "start"],
        "daemon process lifecycle, not a GitBackend operation",
    ),
    (
        &["daemon", "stop"],
        "daemon process lifecycle, not a GitBackend operation",
    ),
    (
        &["daemon", "restart"],
        "daemon process lifecycle, not a GitBackend operation",
    ),
    (
        &["daemon", "status"],
        "daemon process lifecycle, not a GitBackend operation",
    ),
    (
        &["rewrite", "script", "trust"],
        "trusting an unsandboxed script is a person's decision; an agent can only run scripts a person trusted",
    ),
    (
        &["config", "set"],
        "an agent must not be able to rewrite its own gating config",
    ),
    (
        &["hosting", "ssh-key", "create"],
        "writes a private key and uploads it; too dangerous to let an agent automate",
    ),
    (
        &["hosting", "login"],
        "takes a secret token on stdin; a token must not pass through an agent's tool-call transcript",
    ),
    (
        &["repo-auth", "set"],
        "which identity a repository authenticates with is the user's decision; an agent could switch a repository to a more privileged token",
    ),
    (
        &["repo-auth", "clear"],
        "which identity a repository authenticates with is the user's decision; an agent could switch a repository to a more privileged token",
    ),
    (
        &["flow", "trust-hooks"],
        "trusting the shell commands a repository commits is a person's decision; an agent can only run hooks a person trusted",
    ),
];

/// The `repo-auth` arguments that carry a command to run (a credential helper,
/// a token command): never an MCP argument.
const REPO_AUTH_COMMAND_ARGS: [&str; 2] = ["command", "helper"];

/// The CLI argument that lets a stage add files the staging guard flags.
const ALLOW_SENSITIVE: &str = "allow_sensitive";

/// Every CLI leaf that isn't excluded, as an MCP tool, for the default
/// `[mcp]` settings.
pub fn tools() -> Vec<ToolSpec> {
    tools_for(&gitpoulpi_config::McpConfig::default())
}

/// [`tools`] for the operator's `[mcp]` table: `stage` offers
/// `allow_sensitive` only when `allow_sensitive_staging` is on. Otherwise
/// an agent cannot pass it, and a refused stage tells it to ask the user.
pub fn tools_for(cfg: &gitpoulpi_config::McpConfig) -> Vec<ToolSpec> {
    let mut specs = all_tools();
    if !cfg.allow_sensitive_staging {
        for spec in specs.iter_mut().filter(|s| s.cli_path == ["stage"]) {
            spec.withheld.push(ALLOW_SENSITIVE);
            if let Some(props) = spec
                .input_schema
                .get_mut("properties")
                .and_then(Value::as_object_mut)
            {
                props.remove(ALLOW_SENSITIVE);
            }
        }
    }
    specs
}

fn all_tools() -> Vec<ToolSpec> {
    let mut root = Cli::command();
    root.build();
    let mut specs = Vec::new();
    for sub in root.get_subcommands() {
        if sub.get_name() == "help" {
            continue;
        }
        let mut path = vec![sub.get_name().to_string()];
        walk(sub, &mut path, &mut specs);
    }
    specs
}

fn walk(cmd: &Command, path: &mut Vec<String>, out: &mut Vec<ToolSpec>) {
    let children: Vec<&Command> = cmd.get_subcommands().collect();
    if children.is_empty() {
        if excluded_reason(path).is_none() {
            out.push(build_spec(cmd, path));
        }
        return;
    }
    for child in children {
        // clap's auto-generated `help` subcommand (and its `help <sub>`
        // children) is not a command: skip it, or it is advertised as a
        // mutation tool.
        if child.get_name() == "help" {
            continue;
        }
        path.push(child.get_name().to_string());
        walk(child, path, out);
        path.pop();
    }
}

/// The reason `path` is excluded, if it is.
pub fn excluded_reason(path: &[String]) -> Option<&'static str> {
    EXCLUDED
        .iter()
        .find(|(excluded, _)| excluded.len() == path.len() && excluded.iter().eq(path.iter()))
        .map(|(_, reason)| *reason)
}

fn build_spec(cmd: &Command, path: &[String]) -> ToolSpec {
    let name = path.join("_").replace('-', "_");
    let description = cmd
        .get_about()
        .map(|s| s.to_string())
        .or_else(|| cmd.get_long_about().map(|s| s.to_string()))
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| format!("gitpoulpi {}", path.join(" ")));
    let cli_path: Vec<String> = path.to_vec();
    let mut input_schema = schema::arg_schema(cmd);
    // A command that signs a repository in is a person's to write: whatever
    // `repo-auth` leaf is ever offered never takes one from an agent.
    let mut withheld = Vec::new();
    if cli_path.first().map(String::as_str) == Some("repo-auth") {
        for id in REPO_AUTH_COMMAND_ARGS {
            withheld.push(id);
            if let Some(props) = input_schema
                .get_mut("properties")
                .and_then(Value::as_object_mut)
            {
                props.remove(id);
            }
        }
    }
    let read_only = policy::is_read_only(&cli_path);
    let destructive = policy::is_destructive(&cli_path);
    ToolSpec {
        name,
        cli_path,
        description,
        input_schema,
        read_only,
        // A read is always safely repeatable; among mutations, only the
        // ones `policy::is_destructive` already lists are annotated
        // idempotent here too (deleting something already deleted, or
        // resetting to where you already are, has no further effect) --
        // everything else defaults to "may not be".
        idempotent: read_only || destructive,
        destructive,
        command: cmd.clone(),
        withheld,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    /// Walks the *real* clap tree and asserts every leaf is either a tool
    /// or named in `EXCLUDED` - the guard against silent drift a new CLI
    /// subcommand would otherwise cause.
    #[test]
    fn every_cli_leaf_is_a_tool_or_excluded() {
        let specs = tools();
        let tool_paths: HashSet<Vec<String>> = specs.iter().map(|s| s.cli_path.clone()).collect();

        let mut root = Cli::command();
        root.build();
        let mut leaves = Vec::new();
        collect_leaves(&root, &mut Vec::new(), &mut leaves);

        for path in leaves {
            assert!(
                tool_paths.contains(&path) || excluded_reason(&path).is_some(),
                "gitpoulpi {} is neither a tool nor in EXCLUDED",
                path.join(" ")
            );
        }
    }

    fn collect_leaves(cmd: &Command, path: &mut Vec<String>, out: &mut Vec<Vec<String>>) {
        for child in cmd.get_subcommands() {
            if child.get_name() == "help" {
                continue;
            }
            path.push(child.get_name().to_string());
            if child.get_subcommands().next().is_some() {
                collect_leaves(child, path, out);
            } else if !path.is_empty() {
                out.push(path.clone());
            }
            path.pop();
        }
    }

    #[cfg(feature = "full")]
    #[test]
    fn excluded_entries_name_real_cli_leaves() {
        let mut root = Cli::command();
        root.build();
        let mut leaves = Vec::new();
        collect_leaves(&root, &mut Vec::new(), &mut leaves);
        for (excluded, reason) in EXCLUDED {
            let path: Vec<String> = excluded.iter().map(|s| s.to_string()).collect();
            assert!(
                leaves.contains(&path),
                "EXCLUDED entry {path:?} ({reason}) does not name a real CLI leaf"
            );
        }
    }

    /// Which identity a repository signs in with is a person's decision: no
    /// tool sets or clears it, the exclusions say why, and the read-only
    /// `show` is a tool, `test` a network one gated like `fetch`.
    #[test]
    fn an_agent_cannot_set_repo_auth() {
        let all = tools();
        for leaf in ["set", "clear"] {
            let path = vec!["repo-auth".to_string(), leaf.to_string()];
            assert!(
                all.iter().all(|t| t.cli_path != path),
                "`repo-auth {leaf}` is a tool"
            );
            assert!(
                excluded_reason(&path).is_some(),
                "`repo-auth {leaf}` is not excluded"
            );
        }
        let by_name = |n: &str| {
            all.iter()
                .find(|t| t.name == n)
                .unwrap_or_else(|| panic!("no tool {n}"))
        };
        assert!(by_name("repo_auth_show").read_only);
        let test = by_name("repo_auth_test");
        assert!(
            !test.read_only && !test.destructive,
            "test is gated like fetch"
        );
        // The set command never reaches an agent, even through the others.
        for t in all.iter().filter(|t| t.cli_path[0] == "repo-auth") {
            let props = t.input_schema["properties"].as_object().unwrap();
            assert!(!props.contains_key("command") && !props.contains_key("helper"));
        }
    }

    /// clap's auto-generated `help` subcommands are not commands.
    #[test]
    fn no_help_subcommand_is_a_tool() {
        for spec in tools() {
            assert!(
                !spec.cli_path.iter().any(|s| s == "help"),
                "`{}` is clap's generated help, not a command",
                spec.name
            );
        }
    }

    #[test]
    fn session_and_refs_commands_are_tools_with_the_right_classification() {
        let by_name = |n: &str| {
            tools()
                .into_iter()
                .find(|s| s.name == n)
                .unwrap_or_else(|| panic!("no tool {n}"))
        };
        assert!(by_name("session_list").read_only);
        for n in ["session_new", "session_rename"] {
            let t = by_name(n);
            assert!(!t.read_only && !t.destructive, "{n} is a plain mutation");
        }
        let delete = by_name("session_delete");
        assert!(!delete.read_only && delete.destructive);
        for n in ["refs_hide", "refs_show", "refs_solo"] {
            let t = by_name(n);
            assert!(!t.read_only && !t.destructive, "{n} is a plain mutation");
        }
        assert!(
            tools().iter().all(|t| t.name != "completions"),
            "completions is not an MCP tool"
        );
    }

    #[test]
    fn tool_names_are_unique_and_snake_case() {
        let specs = tools();
        let mut seen = HashSet::new();
        for spec in &specs {
            assert!(
                spec.name
                    .chars()
                    .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_'),
                "`{}` is not snake_case",
                spec.name
            );
            assert!(
                seen.insert(spec.name.clone()),
                "duplicate tool name `{}`",
                spec.name
            );
        }
        assert!(!specs.is_empty());
    }

    #[test]
    fn every_tool_schema_is_valid_json_schema_object() {
        for spec in tools() {
            assert_eq!(
                spec.input_schema.get("type").and_then(|v| v.as_str()),
                Some("object"),
                "{}: input_schema must be a JSON Schema object",
                spec.name
            );
            assert!(
                spec.input_schema.get("properties").is_some(),
                "{}: input_schema must declare properties",
                spec.name
            );
        }
    }

    #[test]
    fn destructive_implies_not_read_only() {
        for spec in tools() {
            if spec.destructive {
                assert!(
                    !spec.read_only,
                    "{} is both destructive and read-only",
                    spec.name
                );
            }
        }
    }

    #[test]
    fn reset_and_discard_are_destructive() {
        let specs = tools();
        let reset = specs.iter().find(|s| s.name == "reset").unwrap();
        let discard = specs.iter().find(|s| s.name == "discard").unwrap();
        assert!(reset.destructive);
        assert!(discard.destructive);
    }
}
