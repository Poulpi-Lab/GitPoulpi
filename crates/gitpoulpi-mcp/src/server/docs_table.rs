//! Docs drift guards: the tool table of `docs/user/mcp-and-agents.md` is
//! generated from the registry, and every CLI leaf must be named in
//! `docs/user/cli-reference.md`, so a new subcommand cannot ship without
//! its documentation. `UPDATE_MCP_DOCS=1 cargo test -p gitpoulpi-mcp
//! docs_table` rewrites the generated table.

use std::path::{Path, PathBuf};

use clap::CommandFactory;

use super::is_open_world;
use crate::{policy, registry};

fn doc_path(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../docs/user")
        .join(name)
}

/// What a tool may do, in the words the docs use.
#[cfg_attr(not(feature = "full"), allow(dead_code))]
fn kind(spec: &registry::ToolSpec) -> String {
    let base = if policy::is_propose(&spec.cli_path) {
        "propose"
    } else if policy::is_view(&spec.cli_path) {
        "view"
    } else if spec.read_only {
        "read"
    } else if spec.destructive {
        "destructive"
    } else {
        "write"
    };
    if is_open_world(&spec.name) {
        format!("{base}, network")
    } else {
        base.to_string()
    }
}

#[cfg_attr(not(feature = "full"), allow(dead_code))]
fn tool_table() -> String {
    let mut specs = registry::tools();
    specs.sort_by(|a, b| a.cli_path.cmp(&b.cli_path));
    let mut out = String::from("| Tool | CLI | Kind |\n|---|---|---|\n");
    for spec in &specs {
        out.push_str(&format!(
            "| `{}` | `gitpoulpi {}` | {} |\n",
            spec.name,
            spec.cli_path.join(" "),
            kind(spec)
        ));
    }
    out.push_str("\nNot exposed as tools:\n\n| CLI | Why |\n|---|---|\n");
    for (path, why) in registry::EXCLUDED {
        out.push_str(&format!("| `gitpoulpi {}` | {} |\n", path.join(" "), why));
    }
    out
}

#[cfg(feature = "full")]
#[test]
fn mcp_tool_table_is_current() {
    let path = doc_path("mcp-and-agents.md");
    let doc = std::fs::read_to_string(&path).unwrap();
    let (begin, end) = ("<!-- mcp-tools:begin -->\n", "<!-- mcp-tools:end -->");
    let from = doc.find(begin).expect("mcp-tools:begin marker") + begin.len();
    let to = doc.find(end).expect("mcp-tools:end marker");
    let table = tool_table();
    if doc[from..to] == table {
        return;
    }
    if std::env::var_os("UPDATE_MCP_DOCS").is_some() {
        std::fs::write(&path, format!("{}{}{}", &doc[..from], table, &doc[to..])).unwrap();
        return;
    }
    panic!(
        "docs/user/mcp-and-agents.md's tool table is stale: run \
         UPDATE_MCP_DOCS=1 cargo test -p gitpoulpi-mcp docs_table"
    );
}

fn cli_leaves(cmd: &clap::Command, path: &mut Vec<String>, out: &mut Vec<String>) {
    let children: Vec<&clap::Command> = cmd
        .get_subcommands()
        .filter(|c| c.get_name() != "help")
        .collect();
    if children.is_empty() {
        out.push(path.join(" "));
        return;
    }
    for child in children {
        path.push(child.get_name().to_string());
        cli_leaves(child, path, out);
        path.pop();
    }
}

/// Every `gitpoulpi` subcommand is named in the CLI reference, as
/// `` `<path> `` followed by a space or a closing backtick.
#[test]
fn every_cli_command_is_in_the_cli_reference() {
    let doc = std::fs::read_to_string(doc_path("cli-reference.md")).unwrap();
    let mut root = gitpoulpi_commands::cli::Cli::command();
    root.build();
    let mut leaves = Vec::new();
    cli_leaves(&root, &mut Vec::new(), &mut leaves);
    let missing: Vec<&String> = leaves
        .iter()
        .filter(|leaf| !doc.contains(&format!("`{leaf} ")) && !doc.contains(&format!("`{leaf}`")))
        .collect();
    assert!(
        missing.is_empty(),
        "docs/user/cli-reference.md does not name these commands: {missing:?}"
    );
}
