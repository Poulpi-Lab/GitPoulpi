//! Without `worktree` / `submodule` built, those subcommands are not in the
//! clap tree at all (so not in `--help`, completions or MCP either).

use clap::CommandFactory;
use gitpoulpi_commands::cli::Cli;

#[test]
#[cfg(not(feature = "worktree"))]
fn no_worktree_subcommand_without_worktree() {
    assert!(Cli::command()
        .get_subcommands()
        .all(|c| c.get_name() != "worktree"));
    assert!(<Cli as clap::Parser>::try_parse_from(["gitpoulpi", "worktree", "list"]).is_err());
}

#[test]
#[cfg(not(feature = "submodule"))]
fn no_submodule_subcommand_without_submodule() {
    assert!(Cli::command()
        .get_subcommands()
        .all(|c| c.get_name() != "submodule"));
    assert!(<Cli as clap::Parser>::try_parse_from(["gitpoulpi", "submodule", "list"]).is_err());
}

/// `clone --recurse-submodules` is a plain git flag and stays in every build.
#[test]
fn clone_recurse_submodules_stays() {
    let root = Cli::command();
    let clone = root.find_subcommand("clone").unwrap();
    assert!(clone
        .get_arguments()
        .any(|a| a.get_long() == Some("recurse-submodules")));
}
