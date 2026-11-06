//! `gitpoulpi mcp serve --stdio` as a real subprocess: proof that the
//! CLI's `Commands::Mcp` arm actually wires up to `gitpoulpi_mcp::serve_stdio`
//! rather than just compiling.

#![cfg(feature = "mcp")]

use std::io::Write;
use std::path::Path;
use std::process::{Command, Stdio};

use serde_json::Value;

fn isolated_config_dir() -> &'static Path {
    static DIR: std::sync::OnceLock<tempfile::TempDir> = std::sync::OnceLock::new();
    DIR.get_or_init(|| tempfile::tempdir().expect("could not create a temp config dir"))
        .path()
}

/// Sends one `initialize` request over stdin and closes it - stdin
/// closing is what makes the process exit on Windows, where there's no
/// SIGTERM to send it instead.
#[test]
fn mcp_serve_stdio_answers_initialize() {
    let mut child = Command::new(env!("CARGO_BIN_EXE_gitpoulpi"))
        .env("GITPOULPI_CONFIG_DIR", isolated_config_dir())
        .args(["--session", "cli-mcp-serve-test"])
        .args(["mcp", "serve", "--stdio"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();

    let request = serde_json::json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "initialize",
        "params": {
            "protocolVersion": "2025-06-18",
            "capabilities": {},
            "clientInfo": { "name": "cli-mcp-test", "version": "0" },
        }
    });
    {
        let stdin = child.stdin.as_mut().unwrap();
        stdin.write_all(format!("{request}\n").as_bytes()).unwrap();
    }
    // Dropping stdin closes it, which is what ends the server's read
    // loop -- there is no other request coming.
    drop(child.stdin.take());

    let output = child
        .wait_with_output()
        .expect("failed to wait for gitpoulpi mcp serve");
    let stdout = String::from_utf8_lossy(&output.stdout);
    let first_line = stdout.lines().next().unwrap_or_else(|| {
        panic!(
            "no output on stdout; stderr:\n{}",
            String::from_utf8_lossy(&output.stderr)
        )
    });
    let response: Value = serde_json::from_str(first_line)
        .unwrap_or_else(|e| panic!("stdout line was not JSON ({e}): {first_line}"));
    assert_eq!(response["id"], Value::from(1));
    assert!(response.get("result").is_some(), "{response}");
    assert_eq!(
        response["result"]["serverInfo"]["name"],
        Value::from("gitpoulpi-mcp")
    );
}

/// `mcp serve --http` on a non-loopback address is refused (nothing is
/// started) unless `--allow-remote` is given.
#[test]
fn mcp_serve_http_refuses_a_non_loopback_address() {
    let output = Command::new(env!("CARGO_BIN_EXE_gitpoulpi"))
        .env("GITPOULPI_CONFIG_DIR", isolated_config_dir())
        .args(["--session", "cli-mcp-http-refuse"])
        .args(["mcp", "serve", "--http", "0.0.0.0:0"])
        .output()
        .unwrap();
    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("--allow-remote"), "{stderr}");
}

/// Drives `mcp serve --stdio` through initialize, `tools/list` and two
/// `tools/call`s, in one process, and returns the responses by id.
fn mcp_conversation(config: &Path, calls: Vec<Value>) -> Vec<Value> {
    mcp_conversation_with(config, &[], calls)
}

fn mcp_conversation_with(config: &Path, extra: &[&str], calls: Vec<Value>) -> Vec<Value> {
    let mut child = Command::new(env!("CARGO_BIN_EXE_gitpoulpi"))
        .env("GITPOULPI_CONFIG_DIR", config)
        .args(["--session", "cli-mcp-session-tools"])
        .args(["mcp", "serve", "--stdio"])
        .args(extra)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    let mut lines = vec![
        serde_json::json!({"jsonrpc":"2.0","id":1,"method":"initialize","params":{
            "protocolVersion":"2025-06-18","capabilities":{},
            "clientInfo":{"name":"t","version":"0"}}}),
        serde_json::json!({"jsonrpc":"2.0","method":"notifications/initialized"}),
    ];
    lines.extend(calls);
    {
        let stdin = child.stdin.as_mut().unwrap();
        for line in lines {
            stdin.write_all(format!("{line}\n").as_bytes()).unwrap();
        }
    }
    drop(child.stdin.take());
    let output = child.wait_with_output().unwrap();
    String::from_utf8_lossy(&output.stdout)
        .lines()
        .filter_map(|l| serde_json::from_str::<Value>(l).ok())
        .collect()
}

/// No `help_*` tools; the session/refs tools are present with the right
/// destructive/read-only annotations; `completions` is not a tool; and
/// the session tools actually run.
#[test]
fn mcp_lists_session_and_refs_tools_without_help_and_runs_session_tools() {
    let config = tempfile::tempdir().unwrap();
    std::fs::write(
        config.path().join("config.toml"),
        "[mcp]\nallow_mutations = true\n",
    )
    .unwrap();
    let call = |id: u64, name: &str, args: Value| {
        serde_json::json!({"jsonrpc":"2.0","id":id,"method":"tools/call",
            "params":{"name":name,"arguments":args}})
    };
    // The server answers requests concurrently, so each call that depends
    // on the previous one's effect gets its own server run.
    let once = |request: Value| {
        let id = request["id"].clone();
        mcp_conversation(config.path(), vec![request])
            .into_iter()
            .find(|r| r["id"] == id)
            .expect("no response")
    };
    let listing = once(serde_json::json!({"jsonrpc":"2.0","id":2,"method":"tools/list"}));
    let created = once(call(
        3,
        "session_new",
        serde_json::json!({"name":"agent-made"}),
    ));
    let renamed = once(call(
        4,
        "session_rename",
        serde_json::json!({"old_name":"agent-made","new_name":"renamed"}),
    ));
    let listed_sessions = once(call(5, "session_list", serde_json::json!({})));
    let deleted = once(call(
        6,
        "session_delete",
        serde_json::json!({"name":"renamed"}),
    ));
    let tools = listing["result"]["tools"].as_array().unwrap().clone();
    let tool = |name: &str| {
        tools
            .iter()
            .find(|t| t["name"] == name)
            .unwrap_or_else(|| panic!("no tool {name}"))
            .clone()
    };
    assert!(
        tools
            .iter()
            .all(|t| !t["name"].as_str().unwrap().starts_with("help")
                && !t["name"].as_str().unwrap().contains("_help")),
        "help tools leaked: {:?}",
        tools.iter().map(|t| t["name"].clone()).collect::<Vec<_>>()
    );
    assert!(tools.iter().all(|t| t["name"] != "completions"));
    assert_eq!(tool("session_list")["annotations"]["readOnlyHint"], true);
    for name in [
        "session_new",
        "session_rename",
        "refs_hide",
        "refs_show",
        "refs_solo",
    ] {
        let a = &tool(name)["annotations"];
        assert_eq!(a["readOnlyHint"], false, "{name}");
        assert_eq!(a["destructiveHint"], false, "{name}");
    }
    assert_eq!(
        tool("session_delete")["annotations"]["destructiveHint"],
        true
    );

    assert_eq!(
        created["result"]["structuredContent"]["created"],
        "agent-made"
    );
    assert_eq!(
        renamed["result"]["structuredContent"]["renamed"]["to"],
        "renamed"
    );
    let listed = &listed_sessions["result"]["structuredContent"]["sessions"];
    assert!(listed
        .as_array()
        .unwrap()
        .iter()
        .any(|s| s["name"] == "renamed"));
    assert_eq!(deleted["result"]["structuredContent"]["deleted"], "renamed");
}

/// `config get` and `config path` are MCP tools: they answer from the
/// config file, with no daemon, instead of panicking in the dispatcher.
#[test]
fn mcp_config_get_and_path_tools_answer() {
    let config = tempfile::tempdir().unwrap();
    std::fs::write(
        config.path().join("config.toml"),
        "[general]\ndefault_backend = \"system\"\n",
    )
    .unwrap();
    let call = |id: u64, name: &str, args: Value| {
        serde_json::json!({"jsonrpc":"2.0","id":id,"method":"tools/call",
            "params":{"name":name,"arguments":args}})
    };
    let responses = mcp_conversation(
        config.path(),
        vec![
            call(2, "config_path", serde_json::json!({})),
            call(
                3,
                "config_get",
                serde_json::json!({"key":"general.default_backend"}),
            ),
        ],
    );
    let by_id = |id: u64| {
        responses
            .iter()
            .find(|r| r["id"] == id)
            .unwrap_or_else(|| panic!("no response for {id}: {responses:?}"))
    };
    let path = &by_id(2)["result"];
    assert_ne!(path["isError"], true, "{path}");
    assert_eq!(path["structuredContent"]["exists"], true);
    assert!(path["structuredContent"]["path"]
        .as_str()
        .unwrap()
        .ends_with("config.toml"));

    let get = &by_id(3)["result"];
    assert_ne!(get["isError"], true, "{get}");
    assert_eq!(get["structuredContent"]["value"], "system");
    assert_eq!(get["structuredContent"]["source"], "file");
}

/// The `ui_*` tools over real stdio: listed as view-only reads, and the
/// operator's `--ui-control off` (an argument of the server, not of the
/// agent's call) refuses the ones that would move the user's view.
#[test]
fn mcp_stdio_lists_the_ui_tools_and_honours_ui_control_off() {
    let config = tempfile::tempdir().unwrap();
    let responses = mcp_conversation_with(
        config.path(),
        &["--ui-control", "off"],
        vec![
            serde_json::json!({"jsonrpc":"2.0","id":2,"method":"tools/list"}),
            serde_json::json!({"jsonrpc":"2.0","id":3,"method":"tools/call",
                "params":{"name":"ui_notify","arguments":{"message":"hi"}}}),
        ],
    );
    let by_id = |id: u64| {
        responses
            .iter()
            .find(|r| r["id"] == id)
            .expect("no response")
    };
    let tools = by_id(2)["result"]["tools"].as_array().unwrap().clone();
    for name in [
        "ui_state",
        "ui_viewers",
        "ui_show_diff",
        "ui_show_file",
        "ui_notify",
    ] {
        let tool = tools
            .iter()
            .find(|t| t["name"] == name)
            .unwrap_or_else(|| panic!("no tool {name}"));
        assert_eq!(tool["annotations"]["readOnlyHint"], true, "{name}");
        assert_eq!(tool["annotations"]["destructiveHint"], false, "{name}");
    }
    let refused = by_id(3);
    assert_eq!(refused["result"]["isError"], true, "{refused}");
    assert!(refused["result"]["structuredContent"]["error"]
        .as_str()
        .unwrap()
        .contains("ui control is off"));
}

/// `mcp.enabled = false` makes the server refuse to start, with a message
/// naming the key.
#[test]
fn mcp_serve_refuses_when_disabled_in_config() {
    let config = tempfile::tempdir().unwrap();
    std::fs::write(
        config.path().join("config.toml"),
        "[mcp]\nenabled = false\n",
    )
    .unwrap();
    for extra in [&["--stdio"][..], &["--http"][..]] {
        let output = Command::new(env!("CARGO_BIN_EXE_gitpoulpi"))
            .env("GITPOULPI_CONFIG_DIR", config.path())
            .args(["--session", "cli-mcp-disabled-test", "mcp", "serve"])
            .args(extra)
            .stdin(Stdio::null())
            .output()
            .unwrap();
        assert!(!output.status.success(), "{extra:?}");
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(
            stderr.contains("mcp.enabled = false"),
            "{extra:?}: {stderr}"
        );
    }
}

/// `show --raw` streams bytes to stdout or writes a file the caller names:
/// the tool refuses it, while plain `show` stays a tool.
#[test]
fn mcp_refuses_show_raw() {
    let config = tempfile::tempdir().unwrap();
    let responses = mcp_conversation(
        config.path(),
        vec![
            serde_json::json!({"jsonrpc":"2.0","id":2,"method":"tools/list"}),
            serde_json::json!({"jsonrpc":"2.0","id":3,"method":"tools/call",
                "params":{"name":"show","arguments":{"path":"a.png","raw":true}}}),
        ],
    );
    let by_id = |id: u64| {
        responses
            .iter()
            .find(|r| r["id"] == id)
            .expect("no response")
    };
    let tools = by_id(2)["result"]["tools"].as_array().unwrap().clone();
    assert!(tools.iter().any(|t| t["name"] == "show"));
    let refused = &by_id(3)["result"];
    assert_eq!(refused["isError"], true);
    assert!(refused["structuredContent"]["error"]
        .as_str()
        .unwrap()
        .contains("--raw"));
}

/// The workspace tools and `search` are generated from the CLI tree: reads
/// run with mutations off, the writes need `allow_mutations`, and the
/// repeatable `repos` / `kinds` arguments arrive as arrays.
#[cfg(all(feature = "workspaces", feature = "global-search"))]
#[test]
fn mcp_runs_the_workspace_and_search_tools() {
    let repo = gitpoulpi_testkit::FixtureRepoBuilder::new()
        .unwrap()
        .commit("fix the parser")
        .unwrap()
        .build();
    let repo_path = repo.root().to_string_lossy().into_owned();
    let call = |id: u64, name: &str, args: Value| {
        serde_json::json!({"jsonrpc":"2.0","id":id,"method":"tools/call",
            "params":{"name":name,"arguments":args}})
    };
    let find = |responses: &[Value], id: u64| {
        responses
            .iter()
            .find(|r| r["id"] == id)
            .expect("no response")
            .clone()
    };

    // Mutations off: the reads work, the writes are refused.
    let locked = tempfile::tempdir().unwrap();
    let responses = mcp_conversation(
        locked.path(),
        vec![
            call(2, "workspace_list", serde_json::json!({})),
            call(3, "workspace_create", serde_json::json!({"name": "Agents"})),
        ],
    );
    assert_eq!(
        find(&responses, 2)["result"]["structuredContent"]["workspaces"],
        serde_json::json!([])
    );
    assert_eq!(find(&responses, 3)["result"]["isError"], true);

    // Mutations on: create with a repository, then search it by workspace.
    let open = tempfile::tempdir().unwrap();
    std::fs::write(
        open.path().join("config.toml"),
        "[mcp]\nallow_mutations = true\n",
    )
    .unwrap();
    let once = |id: u64, name: &str, args: Value| {
        find(
            &mcp_conversation(open.path(), vec![call(id, name, args)]),
            id,
        )
    };
    let created = once(
        2,
        "workspace_create",
        serde_json::json!({"name": "Agents", "repos": [repo_path]}),
    );
    assert_eq!(
        created["result"]["structuredContent"]["workspace"]["id"], "agents",
        "{created}"
    );
    let found = once(
        3,
        "search",
        serde_json::json!({"query": "parser", "workspace": "agents", "kinds": ["commit"]}),
    );
    let hits = found["result"]["structuredContent"]["hits"]
        .as_array()
        .unwrap_or_else(|| panic!("{found}"));
    assert_eq!(hits.len(), 1, "{found}");
    assert_eq!(hits[0]["label"], "fix the parser");
    let deleted = once(4, "workspace_delete", serde_json::json!({"id": "agents"}));
    assert_eq!(deleted["result"]["structuredContent"]["deleted"], "agents");
}

// No test reads or writes the real config, data dir or credentials of whoever runs it.
gitpoulpi_testkit::isolate_binary!();

/// Runs `gitpoulpi mcp ...` against its own temporary config folder.
fn mcp_in(dir: &Path, args: &[&str]) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_gitpoulpi"))
        .env("GITPOULPI_CONFIG_DIR", dir)
        .args(["--session", "cli-mcp-token"])
        .arg("mcp")
        .args(args)
        .output()
        .unwrap()
}

fn json_of(output: &std::process::Output) -> Value {
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    serde_json::from_slice(&output.stdout).expect("stdout is JSON")
}

/// `mcp token`: masked by default, the full token only with `--show`,
/// replaced by `--rotate`; the file is the one the server reads.
#[test]
fn mcp_token_reports_a_hint_and_only_shows_the_token_when_asked() {
    let dir = tempfile::tempdir().unwrap();
    let plain = json_of(&mcp_in(dir.path(), &["token"]));
    assert!(plain.get("token").is_none(), "{plain}");
    assert_eq!(plain["rotated"], Value::Bool(false));
    let path = plain["path"].as_str().unwrap();
    assert!(path.ends_with("mcp-http.token"), "{path}");
    let on_disk = std::fs::read_to_string(path).unwrap();

    let shown = json_of(&mcp_in(dir.path(), &["token", "--show"]));
    assert_eq!(shown["token"].as_str().unwrap(), on_disk.trim());
    assert_eq!(shown["token_hint"], plain["token_hint"]);

    let rotated = json_of(&mcp_in(dir.path(), &["token", "--rotate", "--show"]));
    assert_eq!(rotated["rotated"], Value::Bool(true));
    assert_ne!(rotated["token"], shown["token"]);
    assert_eq!(
        std::fs::read_to_string(path).unwrap().trim(),
        rotated["token"].as_str().unwrap()
    );
}

/// `--allow-remote` with the defaults (no TLS, no proxy) is refused and the
/// refusal names what is missing.
#[test]
fn mcp_serve_allow_remote_names_what_is_missing() {
    let dir = tempfile::tempdir().unwrap();
    let output = mcp_in(
        dir.path(),
        &["serve", "--http", "0.0.0.0:0", "--allow-remote"],
    );
    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("unencrypted"), "{stderr}");
    assert!(stderr.contains("mcp.tls"), "{stderr}");
}
