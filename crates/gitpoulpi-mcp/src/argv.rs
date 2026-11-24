//! Turns one MCP `tools/call`'s JSON arguments back into the argv the
//! `gitpoulpi` binary would have parsed for the equivalent command line,
//! so `Cli::try_parse_from` -- clap itself -- does the validation, rather
//! than a second, hand-written checker that could disagree with it.

use clap::{ArgAction, Parser};
use gitpoulpi_commands::cli::Cli;
use serde_json::{Map, Value};

use crate::registry::ToolSpec;

/// `["gitpoulpi", "--json", "--repo", ..., <cli_path...>, <args...>]`.
///
/// `args` is validated against `spec`'s own declared arguments (plus the
/// always-present `repo`/`session`): an unrecognized property is refused
/// here rather than silently ignored or passed through to clap as a
/// stray positional it would misinterpret.
pub fn to_argv(spec: &ToolSpec, args: &Map<String, Value>) -> Result<Vec<String>, String> {
    let mut known: std::collections::HashSet<&str> = spec
        .command
        .get_arguments()
        .map(|a| a.get_id().as_str())
        .filter(|id| !["help", "version", "trust", "hash"].contains(id))
        .filter(|id| !spec.withheld.iter().any(|w| w == id))
        .collect();
    known.insert("repo");
    known.insert("session");

    if let Some(unknown) = args.keys().find(|k| !known.contains(k.as_str())) {
        return Err(format!(
            "unknown argument `{unknown}` for tool `{}`",
            spec.name
        ));
    }

    let mut argv = vec!["gitpoulpi".to_string(), "--json".to_string()];
    if let Some(repo) = args.get("repo").and_then(Value::as_str) {
        argv.push("--repo".to_string());
        argv.push(repo.to_string());
    }
    if let Some(session) = args.get("session").and_then(Value::as_str) {
        argv.push("--session".to_string());
        argv.push(session.to_string());
    }
    argv.extend(spec.cli_path.iter().cloned());

    // Positionals first (in clap's declared order), then flags -- clap
    // expects positionals at the argv positions it declared them, so
    // interleaving them with flags after the fact would risk one being
    // consumed as the other's value.
    let mut positionals = Vec::new();
    let mut options = Vec::new();
    for arg in spec.command.get_arguments() {
        let id = arg.get_id().as_str();
        if id == "help" || id == "version" {
            continue;
        }
        if arg.is_positional() {
            positionals.push(arg);
        } else {
            options.push(arg);
        }
    }

    for arg in positionals {
        let id = arg.get_id().as_str();
        match args.get(id) {
            Some(value) => push_positional(&mut argv, value)?,
            None if arg.is_required_set() => {
                return Err(format!("missing required argument `{id}`"))
            }
            None => {}
        }
    }

    for arg in options {
        let id = arg.get_id().as_str();
        let Some(value) = args.get(id) else {
            if arg.is_required_set() {
                return Err(format!("missing required argument `{id}`"));
            }
            continue;
        };
        let flag = format!("--{}", arg.get_long().unwrap_or(id));
        match arg.get_action() {
            ArgAction::SetTrue => {
                if value.as_bool().unwrap_or(false) {
                    argv.push(flag);
                }
            }
            ArgAction::Append => {
                let items: Vec<Value> = value
                    .as_array()
                    .cloned()
                    .unwrap_or_else(|| vec![value.clone()]);
                for item in items {
                    argv.push(flag.clone());
                    argv.push(scalar_to_string(&item)?);
                }
            }
            _ => {
                argv.push(flag);
                argv.push(scalar_to_string(value)?);
            }
        }
    }

    Ok(argv)
}

/// `argv` parsed by clap, exactly as a real invocation would be - the
/// one point where malformed arguments turn into the same error clap
/// itself would report for a bad command line.
pub fn parse(argv: &[String]) -> Result<Cli, String> {
    Cli::try_parse_from(argv).map_err(|e| e.to_string())
}

fn push_positional(argv: &mut Vec<String>, value: &Value) -> Result<(), String> {
    if let Some(items) = value.as_array() {
        for item in items {
            argv.push(scalar_to_string(item)?);
        }
    } else {
        argv.push(scalar_to_string(value)?);
    }
    Ok(())
}

fn scalar_to_string(value: &Value) -> Result<String, String> {
    match value {
        Value::String(s) => Ok(s.clone()),
        Value::Number(n) => Ok(n.to_string()),
        Value::Bool(b) => Ok(b.to_string()),
        other => Err(format!("`{other}` is not a string, number or boolean")),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::registry;
    use serde_json::json;

    fn spec(name: &str) -> ToolSpec {
        registry::tools()
            .into_iter()
            .find(|s| s.name == name)
            .unwrap_or_else(|| panic!("no such tool: {name}"))
    }

    #[test]
    fn bool_flags_become_bare_switches() {
        let s = spec("log");
        let args: Map<String, Value> = json!({ "graph": true }).as_object().unwrap().clone();
        let argv = to_argv(&s, &args).unwrap();
        assert!(argv.contains(&"--graph".to_string()));

        let args_false: Map<String, Value> = json!({ "graph": false }).as_object().unwrap().clone();
        let argv_false = to_argv(&s, &args_false).unwrap();
        assert!(!argv_false.contains(&"--graph".to_string()));
    }

    #[test]
    fn arrays_repeat_the_flag() {
        let s = spec("log");
        let args: Map<String, Value> = json!({ "path": ["a.txt", "b.txt"] })
            .as_object()
            .unwrap()
            .clone();
        let argv = to_argv(&s, &args).unwrap();
        let flag_count = argv.iter().filter(|a| *a == "--path").count();
        assert_eq!(flag_count, 2);
        assert!(argv
            .windows(2)
            .any(|w| w == ["--path".to_string(), "a.txt".to_string()]));
        assert!(argv
            .windows(2)
            .any(|w| w == ["--path".to_string(), "b.txt".to_string()]));
    }

    /// Agents cannot grant trust to an unsandboxed script: there is no tool
    /// for `rewrite script trust`, and `--trust` is neither in any schema nor
    /// accepted as an argument.
    #[test]
    fn an_agent_cannot_trust_a_script() {
        assert!(
            registry::tools()
                .iter()
                .all(|s| s.cli_path != ["rewrite", "script", "trust"]),
            "`rewrite script trust` is a tool"
        );
        // Without the `history-rewrite` feature there are no rewrite tools.
        if !registry::tools()
            .iter()
            .any(|s| s.name == "rewrite_preview")
        {
            return;
        }
        for tool in [
            "rewrite_preview",
            "rewrite_run",
            "rewrite_script_check",
            "rewrite_script_test",
        ] {
            let s = spec(tool);
            let schema = crate::schema::arg_schema(&s.command);
            assert!(
                schema["properties"].get("trust").is_none(),
                "{tool} offers --trust"
            );
            let args: Map<String, Value> = json!({ "trust": true }).as_object().unwrap().clone();
            assert!(to_argv(&s, &args).is_err(), "{tool} accepted trust");
        }
        // The rest of the recipe flags are still there.
        let s = spec("rewrite_preview");
        let schema = crate::schema::arg_schema(&s.command);
        assert!(schema["properties"].get("script_language").is_some());
    }

    /// Agents cannot grant trust to a repository's committed workflow hooks:
    /// there is no tool for `flow trust-hooks`, and a `hash` argument is
    /// neither in any schema nor accepted.
    #[test]
    fn an_agent_cannot_trust_workflow_hooks() {
        assert!(
            registry::tools()
                .iter()
                .all(|s| s.cli_path != ["flow", "trust-hooks"]),
            "`flow trust-hooks` is a tool"
        );
        assert!(
            registry::excluded_reason(&["flow".to_string(), "trust-hooks".to_string()]).is_some()
        );
        let s = spec("flow_start");
        let schema = crate::schema::arg_schema(&s.command);
        assert!(schema["properties"].get("hash").is_none());
        assert!(schema["properties"].get("run_hooks").is_some());
        let args: Map<String, Value> = json!({ "hash": "00", "type": "feature" })
            .as_object()
            .unwrap()
            .clone();
        assert!(to_argv(&s, &args).is_err(), "flow_start accepted a hash");
    }

    /// Agents cannot trust a repository's own sequences either (the same
    /// precedent as the workflow hooks), but they can propose running one:
    /// `ui_sequence_run` is a tool of the propose class.
    #[test]
    fn an_agent_cannot_trust_repository_sequences_but_can_propose_a_run() {
        for leaf in ["trust", "untrust"] {
            assert!(
                registry::tools()
                    .iter()
                    .all(|s| s.cli_path != ["sequence", leaf]),
                "`sequence {leaf}` is a tool"
            );
            assert!(
                registry::excluded_reason(&["sequence".to_string(), leaf.to_string()]).is_some()
            );
        }
        let run = spec("ui_sequence_run");
        assert!(crate::policy::is_propose(&run.cli_path));
    }

    /// `stage` offers `allow_sensitive` only when the operator turned
    /// `[mcp] allow_sensitive_staging` on: otherwise it is neither in the
    /// schema nor accepted, so an agent cannot pass the staging guard.
    #[test]
    fn allow_sensitive_is_offered_only_when_the_operator_allows_it() {
        let stage = |allow: bool| {
            let cfg = gitpoulpi_config::McpConfig {
                allow_sensitive_staging: allow,
                ..Default::default()
            };
            registry::tools_for(&cfg)
                .into_iter()
                .find(|s| s.name == "stage")
                .expect("stage is a tool")
        };
        let args: Map<String, Value> = json!({ "all": true, "allow_sensitive": true })
            .as_object()
            .unwrap()
            .clone();

        let closed = stage(false);
        assert!(closed.input_schema["properties"]
            .get("allow_sensitive")
            .is_none());
        assert!(closed.input_schema["properties"].get("all").is_some());
        let err = to_argv(&closed, &args).unwrap_err();
        assert!(err.contains("allow_sensitive"), "{err}");
        // A plain stage still works.
        let plain: Map<String, Value> = json!({ "all": true }).as_object().unwrap().clone();
        assert!(to_argv(&closed, &plain).is_ok());

        let open = stage(true);
        assert!(open.input_schema["properties"]
            .get("allow_sensitive")
            .is_some());
        let argv = to_argv(&open, &args).unwrap();
        assert!(argv.contains(&"--allow-sensitive".to_string()), "{argv:?}");
        // The default table is the closed one.
        assert!(spec("stage").withheld.contains(&"allow_sensitive"));
    }

    #[test]
    fn unknown_property_is_invalid_params() {
        let s = spec("status");
        let args: Map<String, Value> = json!({ "nope": true }).as_object().unwrap().clone();
        assert!(to_argv(&s, &args).is_err());
    }

    #[test]
    fn a_full_round_trip_parses_with_clap() {
        let s = spec("branch_create");
        let args: Map<String, Value> = json!({ "name": "feature/x" }).as_object().unwrap().clone();
        let argv = to_argv(&s, &args).unwrap();
        let cli = parse(&argv).unwrap();
        assert!(matches!(
            cli.command,
            Some(gitpoulpi_commands::cli::Commands::Branch { .. })
        ));
    }

    #[test]
    fn missing_required_argument_is_an_error() {
        let s = spec("checkout");
        let args: Map<String, Value> = Map::new();
        assert!(to_argv(&s, &args).is_err());
    }
}

#[cfg(all(test, feature = "ai"))]
mod ai_limit_tests {
    use super::*;
    use crate::registry;
    use serde_json::json;

    /// The AI tools take `send_all` (the CLI's `--send-all`): an agent that
    /// cannot answer a dialog can still ask for every input to be sent whole.
    /// The `truncated` field of the answer comes from the shared command layer.
    #[test]
    fn the_ai_tools_take_send_all_and_it_reaches_clap() {
        let tool = |name: &str| {
            registry::tools()
                .into_iter()
                .find(|s| s.name == name)
                .unwrap_or_else(|| panic!("no such tool: {name}"))
        };
        for name in [
            "ai_commit_message",
            "ai_explain",
            "ai_resolve_conflict",
            "ai_split_plan",
            "hosting_review_suggest",
            "commit",
            "stash_push",
        ] {
            let schema = crate::schema::arg_schema(&tool(name).command);
            assert_eq!(
                schema["properties"]["send_all"]["type"], "boolean",
                "{name}: {schema}"
            );
        }
        let args: Map<String, Value> = json!({ "send_all": true }).as_object().unwrap().clone();
        let argv = to_argv(&tool("ai_commit_message"), &args).unwrap();
        assert!(argv.contains(&"--send-all".to_string()), "{argv:?}");
        assert!(parse(&argv).is_ok());
    }
}
