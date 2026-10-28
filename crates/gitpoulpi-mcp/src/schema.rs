//! JSON Schema for one tool's arguments, derived from clap metadata --
//! clap is the one source of truth for what a subcommand accepts, so
//! `gitpoulpi-core` never has to grow a `schemars` dependency of its own.

use clap::{Arg, ArgAction, Command};
use serde_json::{json, Value};

/// The `input_schema` for `cmd` (a CLI leaf): one property per declared
/// argument, plus `repo` and `session`, which every tool accepts even
/// though no CLI leaf declares them itself (they are `Cli`'s own global
/// flags - see `argv.rs`).
pub fn arg_schema(cmd: &Command) -> Value {
    let mut properties = serde_json::Map::new();
    let mut required = Vec::new();

    for arg in cmd.get_arguments() {
        let id = arg.get_id().as_str();
        // `lang` only picks the language of the CLI's `--help`; MCP output
        // stays English, so it is not a tool argument.
        if id == "help" || id == "version" || id == "lang" {
            continue;
        }
        // `--trust` remembers an unsandboxed script as trusted: a person's
        // decision, so it is not a tool argument (argv.rs refuses it too).
        if id == "trust" {
            continue;
        }
        let (schema, is_required) = single_arg_schema(arg);
        properties.insert(id.to_string(), schema);
        if is_required {
            required.push(id.to_string());
        }
    }

    properties.insert(
        "repo".to_string(),
        json!({
            "type": "string",
            "description": "Repository to act on. Defaults to the repository enclosing the MCP server's own working directory."
        }),
    );
    properties.insert(
        "session".to_string(),
        json!({
            "type": "string",
            "description": "Named session whose daemon to use. Defaults to (and today can only be) the server's own session."
        }),
    );

    let mut schema = json!({
        "type": "object",
        "properties": Value::Object(properties),
        "additionalProperties": false,
    });
    if !required.is_empty() {
        schema["required"] = json!(required);
    }
    schema
}

fn single_arg_schema(arg: &Arg) -> (Value, bool) {
    let required = arg.is_required_set();

    let possible: Vec<String> = arg
        .get_possible_values()
        .iter()
        .map(|p| p.get_name().to_string())
        .collect();

    let scalar = if !possible.is_empty() {
        json!({ "type": "string", "enum": possible })
    } else if matches!(arg.get_action(), ArgAction::SetTrue | ArgAction::SetFalse) {
        json!({ "type": "boolean" })
    } else if is_integer(arg) {
        json!({ "type": "integer" })
    } else {
        json!({ "type": "string" })
    };

    let schema = if matches!(arg.get_action(), ArgAction::Append) {
        json!({ "type": "array", "items": scalar })
    } else {
        scalar
    };

    if let Some(help) = arg.get_help() {
        let mut schema = schema;
        schema["description"] = json!(help.to_string());
        return (schema, required);
    }
    (schema, required)
}

/// `u32`/`usize` value-parsed args (`--limit`, `--skip`, `--hunk`,
/// `--depth`) render as `integer` rather than the generic `string`
/// fallback. Anything clap doesn't parse as one of these two stays a
/// string - true for every other argument in the CLI today.
fn is_integer(arg: &Arg) -> bool {
    let u32_id = clap::builder::ValueParser::from(clap::value_parser!(u32)).type_id();
    let usize_id = clap::builder::ValueParser::from(clap::value_parser!(usize)).type_id();
    let id = arg.get_value_parser().type_id();
    id == u32_id || id == usize_id
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::CommandFactory;
    use gitpoulpi_commands::cli::Cli;

    fn leaf<'a>(root: &'a Command, path: &[&str]) -> &'a Command {
        let mut cmd = root;
        for segment in path {
            cmd = cmd
                .get_subcommands()
                .find(|c| c.get_name() == *segment)
                .unwrap_or_else(|| panic!("no such subcommand: {segment}"));
        }
        cmd
    }

    #[test]
    fn every_schema_carries_repo_and_session() {
        let mut root = Cli::command();
        root.build();
        let schema = arg_schema(leaf(&root, &["status"]));
        let props = schema["properties"].as_object().unwrap();
        assert!(props.contains_key("repo"));
        assert!(props.contains_key("session"));
    }

    #[test]
    fn a_bool_flag_is_a_boolean() {
        let mut root = Cli::command();
        root.build();
        let schema = arg_schema(leaf(&root, &["log"]));
        assert_eq!(schema["properties"]["graph"]["type"], "boolean");
    }

    #[test]
    fn a_repeatable_flag_is_an_array() {
        let mut root = Cli::command();
        root.build();
        let schema = arg_schema(leaf(&root, &["log"]));
        assert_eq!(schema["properties"]["path"]["type"], "array");
    }

    #[test]
    fn a_numeric_flag_is_an_integer() {
        let mut root = Cli::command();
        root.build();
        let schema = arg_schema(leaf(&root, &["log"]));
        assert_eq!(schema["properties"]["limit"]["type"], "integer");
    }

    #[test]
    fn a_value_enum_is_a_string_enum() {
        let mut root = Cli::command();
        root.build();
        let schema = arg_schema(leaf(&root, &["reset"]));
        let mode = &schema["properties"]["mode"];
        assert_eq!(mode["type"], "string");
        assert!(mode["enum"].as_array().unwrap().contains(&json!("soft")));
    }

    #[test]
    fn a_required_positional_is_listed_in_required() {
        let mut root = Cli::command();
        root.build();
        let schema = arg_schema(leaf(&root, &["checkout"]));
        let required: Vec<&str> = schema["required"]
            .as_array()
            .unwrap()
            .iter()
            .map(|v| v.as_str().unwrap())
            .collect();
        assert!(required.contains(&"target"));
    }

    /// The workflow actions are plan-then-execute for an agent: `dry_run`
    /// shows the plan, `yes` runs it (neither: the plan comes back as a
    /// refusal), `run_hooks` is the explicit opt-in for the hook commands.
    #[test]
    fn the_flow_actions_expose_plan_confirmation_and_hooks() {
        let mut root = Cli::command();
        root.build();
        for action in ["start", "finish", "sync", "promote", "set-parent"] {
            let schema = arg_schema(leaf(&root, &["flow", action]));
            for name in ["dry_run", "yes", "run_hooks", "no_hooks"] {
                assert_eq!(
                    schema["properties"][name]["type"], "boolean",
                    "flow {action} {name}"
                );
            }
        }
        let schema = arg_schema(leaf(&root, &["flow", "finish"]));
        assert_eq!(schema["properties"]["continue_"]["type"], "boolean");
        assert_eq!(schema["properties"]["abort"]["type"], "boolean");
        let start = arg_schema(leaf(&root, &["flow", "start"]));
        let required: Vec<&str> = start["required"]
            .as_array()
            .unwrap()
            .iter()
            .map(|v| v.as_str().unwrap())
            .collect();
        assert!(required.contains(&"type"));
    }
}
