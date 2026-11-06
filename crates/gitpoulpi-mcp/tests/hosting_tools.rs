//! MCP tools follow the clap tree, so a binary built without a hosting
//! provider has no `hosting_*` tool, and one built with it keeps them.

use gitpoulpi_mcp::registry::tools;

fn hosting_tools() -> usize {
    tools()
        .iter()
        .filter(|t| t.cli_path.first().map(String::as_str) == Some("hosting"))
        .count()
}

#[cfg(not(any(feature = "github", feature = "gitlab")))]
#[test]
fn no_hosting_tools_without_a_provider() {
    assert_eq!(hosting_tools(), 0);
}

#[cfg(any(feature = "github", feature = "gitlab"))]
#[test]
fn hosting_tools_present_with_a_provider() {
    assert!(hosting_tools() > 0);
}

#[cfg(any(feature = "github", feature = "gitlab"))]
#[test]
fn the_board_commands_are_tools() {
    let names: Vec<String> = tools().into_iter().map(|t| t.name).collect();
    for tool in ["hosting_board_show", "hosting_board_move"] {
        assert!(names.iter().any(|n| n == tool), "{tool} in {names:?}");
    }
}

/// Accepting a changed address, account or certificate is a person's
/// decision: neither trust command is a tool, and both are named in
/// `EXCLUDED`, so an agent only ever reads the refusal.
#[test]
fn an_agent_cannot_trust_an_integration() {
    use gitpoulpi_mcp::registry::excluded_reason;
    let all = tools();
    for path in [
        &["hosting", "integrations", "trust"][..],
        &["ai", "trust"][..],
    ] {
        assert!(
            all.iter().all(|t| t.cli_path != path),
            "`{}` is a tool",
            path.join(" ")
        );
        let owned: Vec<String> = path.iter().map(|s| s.to_string()).collect();
        assert!(
            excluded_reason(&owned).is_some(),
            "`{}` is not excluded",
            path.join(" ")
        );
    }
}

// No test reads or writes the real config, data dir or credentials of whoever runs it.
gitpoulpi_testkit::isolate_binary!();
