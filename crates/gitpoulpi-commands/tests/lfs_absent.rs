//! Without LFS built, the `lfs` subcommand and `clone --no-lfs` are not in
//! the clap tree at all (so not in `--help`, completions or MCP either).

#![cfg(not(feature = "lfs"))]

use clap::{CommandFactory, Parser};
use gitpoulpi_commands::cli::Cli;

#[test]
fn no_lfs_subcommand_or_clone_flag_without_lfs() {
    let root = Cli::command();
    assert!(root.get_subcommands().all(|c| c.get_name() != "lfs"));
    assert!(Cli::try_parse_from(["gitpoulpi", "lfs", "status"]).is_err());
    let clone = root.find_subcommand("clone").unwrap();
    assert!(clone.get_arguments().all(|a| a.get_id() != "no_lfs"));
    assert!(Cli::try_parse_from(["gitpoulpi", "clone", "u", "--no-lfs"]).is_err());
}
