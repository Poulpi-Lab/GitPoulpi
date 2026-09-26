//! With no hosting provider built, the `hosting` subcommand is not in the
//! clap tree at all (so not in `--help`, completions or MCP either).

#![cfg(not(feature = "hosting"))]

use clap::{CommandFactory, Parser};
use gitpoulpi_commands::cli::Cli;

#[test]
fn no_hosting_subcommand_without_a_provider() {
    let root = Cli::command();
    assert!(root.get_subcommands().all(|c| c.get_name() != "hosting"));
    assert!(Cli::try_parse_from(["gitpoulpi", "hosting", "integrations", "list"]).is_err());
}
