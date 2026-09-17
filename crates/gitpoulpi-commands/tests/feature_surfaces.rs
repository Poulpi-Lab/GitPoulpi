//! The History workbench, the global search and the workspaces are features a
//! custom build leaves out: with one off, its subcommands and flags are not in
//! the clap tree at all (so not in `--help`, completions or the MCP tools,
//! which are generated from the tree), and with it on they are there (so the
//! absence checks cannot pass for lack of a tree).

use clap::{Command, CommandFactory, Parser};
use gitpoulpi_commands::cli::Cli;

fn has_subcommand(cmd: &Command, name: &str) -> bool {
    cmd.get_subcommands().any(|c| c.get_name() == name)
}

fn has_arg(cmd: &Command, id: &str) -> bool {
    cmd.get_arguments().any(|a| a.get_id() == id)
}

#[test]
fn the_history_workbench_commands_follow_the_feature() {
    let root = Cli::command();
    let on = cfg!(feature = "history-rewrite");
    assert_eq!(has_subcommand(&root, "rewrite"), on);
    assert_eq!(
        Cli::try_parse_from(["gitpoulpi", "rewrite", "tools"]).is_ok(),
        on
    );
    if let Some(ai) = root.find_subcommand("ai") {
        assert_eq!(has_subcommand(ai, "rewrite-script"), on);
        assert_eq!(has_subcommand(ai, "rewrite-rules"), on);
    }
    if let Some(propose) = root
        .find_subcommand("ui")
        .and_then(|ui| ui.find_subcommand("propose"))
    {
        assert_eq!(has_subcommand(propose, "rewrite"), on);
    }
    // The AI action keys the other commands accept.
    for action in [
        gitpoulpi_config::AiAction::RewriteScript,
        gitpoulpi_config::AiAction::RewriteRules,
    ] {
        assert_eq!(
            gitpoulpi_commands::build_features::ai_action_offered(action),
            on
        );
    }
    assert!(gitpoulpi_commands::build_features::ai_action_offered(
        gitpoulpi_config::AiAction::CommitMessage
    ));
}

#[test]
fn the_global_search_command_follows_the_feature() {
    let root = Cli::command();
    let on = cfg!(feature = "global-search");
    assert_eq!(has_subcommand(&root, "search"), on);
    assert_eq!(
        Cli::try_parse_from(["gitpoulpi", "search", "x"]).is_ok(),
        on
    );
    // `ui show search` is the Log find bar, not the global search.
    assert!(root
        .find_subcommand("ui")
        .and_then(|ui| ui.find_subcommand("show"))
        .is_some_and(|show| has_subcommand(show, "search")));
}

#[test]
fn the_workspace_commands_and_flags_follow_the_feature() {
    let root = Cli::command();
    let on = cfg!(feature = "workspaces");
    assert_eq!(has_subcommand(&root, "workspace"), on);
    assert_eq!(
        Cli::try_parse_from(["gitpoulpi", "workspace", "list"]).is_ok(),
        on
    );
    let commit = root.find_subcommand("commit").unwrap();
    assert_eq!(has_arg(commit, "workspace"), on);
    if let Some(search) = root.find_subcommand("search") {
        assert_eq!(has_arg(search, "workspace"), on);
    }
}

#[test]
fn the_version_line_names_the_three_features_as_built() {
    let flags = gitpoulpi_commands::build_features::FLAGS;
    for (name, built) in [
        ("history-rewrite", cfg!(feature = "history-rewrite")),
        ("global-search", cfg!(feature = "global-search")),
        ("workspaces", cfg!(feature = "workspaces")),
    ] {
        assert_eq!(
            flags.iter().find(|(n, _)| *n == name).map(|(_, b)| *b),
            Some(built),
            "{name}"
        );
    }
}
