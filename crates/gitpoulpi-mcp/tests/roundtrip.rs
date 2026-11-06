//! End-to-end tests over an in-process JSON-RPC pipe: no real stdio or
//! `gitpoulpi-mcp` process, but the real `rmcp::ServerHandler`, the real
//! tool registry, and a real fixture repository through a real embedded
//! daemon -- the same path a `claude mcp add gitpoulpi -- gitpoulpi mcp
//! serve --stdio` client drives.
//!
//! Framing is hand-rolled (newline-delimited JSON-RPC, matching `rmcp`'s
//! stdio transport) so these tests do not need rmcp's `client` feature.

use std::sync::OnceLock;

use gitpoulpi_config::{DefaultBackend, McpConfig};
use gitpoulpi_testkit::FixtureRepoBuilder;
use serde_json::{json, Value};
use tempfile::TempDir;
use tokio::io::{
    AsyncBufReadExt, AsyncWriteExt, BufReader, DuplexStream, Lines, ReadHalf, WriteHalf,
};

/// One throwaway config dir for this whole test binary - see
/// `gitpoulpi-cli/tests/commands.rs`'s twin of this helper for why: every
/// case that starts an embedded daemon must not read the developer's own
/// `config.toml`.
fn isolated_config_dir() -> &'static std::path::Path {
    static DIR: OnceLock<TempDir> = OnceLock::new();
    DIR.get_or_init(|| tempfile::tempdir().expect("could not create a temp config dir"))
        .path()
}

struct TestClient {
    write: WriteHalf<DuplexStream>,
    lines: Lines<BufReader<ReadHalf<DuplexStream>>>,
    next_id: u64,
}

impl TestClient {
    async fn request(&mut self, method: &str, params: Value) -> Value {
        let id = self.next_id;
        self.next_id += 1;
        let request = json!({ "jsonrpc": "2.0", "id": id, "method": method, "params": params });
        self.write
            .write_all(format!("{request}\n").as_bytes())
            .await
            .expect("failed to write request");
        loop {
            let line = self
                .lines
                .next_line()
                .await
                .expect("failed to read a line")
                .expect("server closed the connection");
            let value: Value = serde_json::from_str(&line).expect("server wrote invalid JSON");
            if value.get("id").and_then(Value::as_u64) == Some(id) {
                return value;
            }
        }
    }

    async fn notify(&mut self, method: &str, params: Value) {
        let notification = json!({ "jsonrpc": "2.0", "method": method, "params": params });
        self.write
            .write_all(format!("{notification}\n").as_bytes())
            .await
            .expect("failed to write notification");
    }

    async fn call_tool(&mut self, name: &str, arguments: Value) -> Value {
        self.request(
            "tools/call",
            json!({ "name": name, "arguments": arguments }),
        )
        .await
    }
}

/// Starts a server on an in-process duplex pipe under `session`, with
/// `cfg`, and completes the MCP handshake. `session` must be unique per
/// test: two tests sharing one would race for the same embedded daemon.
/// Mutating tools are off by default (`allow_mutations = false`); the tests
/// that commit or check argument validation on a mutating tool opt in.
fn mutations_on() -> McpConfig {
    McpConfig {
        allow_mutations: true,
        ..McpConfig::default()
    }
}

async fn start_server(session: &str, cfg: McpConfig) -> TestClient {
    std::env::set_var("GITPOULPI_CONFIG_DIR", isolated_config_dir());

    let (client_io, server_io) = tokio::io::duplex(1 << 20);
    let (server_read, server_write) = tokio::io::split(server_io);
    let session_owned = session.to_string();
    tokio::spawn(async move {
        let _ = gitpoulpi_mcp::serve(
            server_read,
            server_write,
            session_owned,
            DefaultBackend::default(),
            cfg,
        )
        .await;
    });

    let (client_read, client_write) = tokio::io::split(client_io);
    let mut client = TestClient {
        write: client_write,
        lines: BufReader::new(client_read).lines(),
        next_id: 1,
    };

    let init = client
        .request(
            "initialize",
            json!({
                "protocolVersion": "2025-06-18",
                "capabilities": {},
                "clientInfo": { "name": "gitpoulpi-mcp-tests", "version": "0" },
            }),
        )
        .await;
    assert!(init.get("result").is_some(), "initialize failed: {init}");
    client.notify("notifications/initialized", json!({})).await;
    client
}

#[tokio::test]
async fn initialize_then_tools_list_has_status_with_read_only_hint() {
    let mut client = start_server("mcp-roundtrip-list", McpConfig::default()).await;
    let response = client.request("tools/list", json!({})).await;
    let tools = response["result"]["tools"]
        .as_array()
        .expect("tools/list must return an array");
    assert!(!tools.is_empty());
    let status = tools
        .iter()
        .find(|t| t["name"] == "status")
        .expect("`status` must be a tool");
    assert_eq!(status["annotations"]["readOnlyHint"], json!(true));

    #[cfg(feature = "lfs")]
    for name in ["lfs_status", "lfs_files"] {
        let tool = tools
            .iter()
            .find(|t| t["name"] == name)
            .unwrap_or_else(|| panic!("`{name}` must be a tool"));
        assert_eq!(tool["annotations"]["readOnlyHint"], json!(true), "{name}");
    }

    let reset = tools
        .iter()
        .find(|t| t["name"] == "reset")
        .expect("`reset` must be a tool");
    assert_eq!(reset["annotations"]["readOnlyHint"], json!(false));
    assert_eq!(reset["annotations"]["destructiveHint"], json!(true));
}

#[tokio::test]
async fn the_job_tools_are_listed_and_job_list_needs_no_repository() {
    let mut client = start_server("mcp-roundtrip-jobs", McpConfig::default()).await;
    let response = client.request("tools/list", json!({})).await;
    let tools = response["result"]["tools"].as_array().unwrap().clone();
    let hint = |name: &str| {
        tools
            .iter()
            .find(|t| t["name"] == name)
            .unwrap_or_else(|| panic!("`{name}` must be a tool"))["annotations"]["readOnlyHint"]
            .clone()
    };
    assert_eq!(hint("job_list"), json!(true));
    assert_eq!(hint("job_wait"), json!(true));
    assert_eq!(hint("job_cancel"), json!(false));

    let response = client.call_tool("job_list", json!({})).await;
    let result = &response["result"];
    assert_ne!(result.get("isError"), Some(&json!(true)), "{result}");
    assert!(
        result["structuredContent"].is_array() || result["structuredContent"]["jobs"].is_array(),
        "{result}"
    );
}

#[tokio::test]
async fn status_on_fixture_returns_working_tree() {
    let repo = FixtureRepoBuilder::new()
        .unwrap()
        .commit("first")
        .unwrap()
        .build();
    let mut client = start_server("mcp-roundtrip-status", McpConfig::default()).await;

    let response = client
        .call_tool("status", json!({ "repo": repo.root().to_string_lossy() }))
        .await;
    let result = &response["result"];
    assert_ne!(result.get("isError"), Some(&json!(true)), "{result}");
    assert!(result["structuredContent"]["files"].is_array(), "{result}");
}

#[tokio::test]
async fn log_returns_a_commit_graph() {
    let repo = FixtureRepoBuilder::new()
        .unwrap()
        .commit("first")
        .unwrap()
        .commit("second")
        .unwrap()
        .build();
    let mut client = start_server("mcp-roundtrip-log", McpConfig::default()).await;

    let response = client
        .call_tool("log", json!({ "repo": repo.root().to_string_lossy() }))
        .await;
    let result = &response["result"];
    assert_ne!(result.get("isError"), Some(&json!(true)), "{result}");
    let nodes = result["structuredContent"]["nodes"]
        .as_array()
        .expect("log must return commit graph nodes");
    assert_eq!(nodes.len(), 2, "{result}");
}

#[tokio::test]
async fn stage_then_commit_then_log_shows_the_new_commit() {
    let repo = FixtureRepoBuilder::new()
        .unwrap()
        .commit("first")
        .unwrap()
        .build();
    std::fs::write(repo.root().join("new.txt"), "hello").unwrap();
    let mut client = start_server("mcp-roundtrip-commit", mutations_on()).await;
    let repo_arg = repo.root().to_string_lossy().to_string();

    let staged = client
        .call_tool("stage", json!({ "repo": repo_arg, "all": true }))
        .await;
    assert_ne!(
        staged["result"].get("isError"),
        Some(&json!(true)),
        "{staged:?}"
    );

    let committed = client
        .call_tool(
            "commit",
            json!({ "repo": repo_arg, "message": "add new.txt" }),
        )
        .await;
    assert_ne!(
        committed["result"].get("isError"),
        Some(&json!(true)),
        "{committed:?}"
    );

    let log = client.call_tool("log", json!({ "repo": repo_arg })).await;
    let nodes = log["result"]["structuredContent"]["nodes"]
        .as_array()
        .unwrap();
    assert_eq!(nodes.len(), 2, "{log:?}");
    assert_eq!(nodes[0]["summary"], json!("add new.txt"));
}

/// An agent drives a workflow action plan-then-execute: `dry_run` shows the
/// plan, neither `dry_run` nor `yes` is refused with the plan (nothing runs),
/// `yes` runs it; hooks stay off until `run_hooks` is passed explicitly.
#[tokio::test]
async fn flow_actions_are_plan_then_execute_and_hooks_need_an_explicit_opt_in() {
    let repo = FixtureRepoBuilder::new()
        .unwrap()
        .commit_file("a.txt", "base\n", "init")
        .unwrap()
        .branch("develop")
        .unwrap()
        .branch("feature/x")
        .unwrap()
        .commit_file("f.txt", "f\n", "f1")
        .unwrap()
        .build();
    let exclude = repo.root().join(".git").join("info").join("exclude");
    std::fs::create_dir_all(exclude.parent().unwrap()).unwrap();
    std::fs::write(exclude, "/.gitpoulpi/\n").unwrap();
    let mut client = start_server("mcp-roundtrip-flow", mutations_on()).await;
    let repo_arg = repo.root().to_string_lossy().to_string();
    let hook = if cfg!(windows) {
        "echo hi> hook.txt"
    } else {
        "echo hi > hook.txt"
    };

    let init = client
        .call_tool(
            "flow_config_init",
            json!({ "repo": repo_arg, "template": "git-flow", "write": true }),
        )
        .await;
    assert_ne!(
        init["result"].get("isError"),
        Some(&json!(true)),
        "{init:?}"
    );
    let file = repo.state_dir().join("workflow.toml");
    let mut text = std::fs::read_to_string(&file).unwrap();
    text.push_str(&format!("\n[hooks]\nbefore_finish = [{hook:?}]\n"));
    std::fs::write(&file, text).unwrap();

    // The plan: a success, the hook listed but not run.
    let plan = client
        .call_tool("flow_finish", json!({ "repo": repo_arg, "dry_run": true }))
        .await;
    assert_ne!(
        plan["result"].get("isError"),
        Some(&json!(true)),
        "{plan:?}"
    );
    let plan = &plan["result"]["structuredContent"]["plan"];
    assert_eq!(plan["title"], "Finish feature/x");
    assert_eq!(plan["hooks_run"], false);
    assert_eq!(plan["hooks"][0]["command"], hook);

    // Neither dry_run nor yes: refused with the plan, nothing happened.
    let refused = client
        .call_tool("flow_finish", json!({ "repo": repo_arg }))
        .await;
    assert_eq!(refused["result"]["isError"], json!(true), "{refused:?}");
    assert!(refused["result"]["structuredContent"]["needs_confirmation"]["plan"].is_object());
    assert!(repo.git(&["branch"]).contains("feature/x"));

    // yes without run_hooks: the merge happens, the hook does not run.
    let done = client
        .call_tool("flow_finish", json!({ "repo": repo_arg, "yes": true }))
        .await;
    assert_ne!(
        done["result"].get("isError"),
        Some(&json!(true)),
        "{done:?}"
    );
    assert_eq!(done["result"]["structuredContent"]["status"], "done");
    assert!(!repo.root().join("hook.txt").exists());
    assert!(!repo.git(&["branch"]).contains("feature/x"));
}

#[tokio::test]
async fn a_bad_revision_is_a_tool_error_not_a_protocol_error() {
    let repo = FixtureRepoBuilder::new()
        .unwrap()
        .commit("first")
        .unwrap()
        .build();
    let mut client = start_server("mcp-roundtrip-badrev", McpConfig::default()).await;

    let response = client
        .call_tool(
            "checkout",
            json!({ "repo": repo.root().to_string_lossy(), "target": "not-a-real-revision" }),
        )
        .await;
    // A protocol-level JSON-RPC error would have no "result" at all.
    assert!(response.get("error").is_none(), "{response:?}");
    assert_eq!(response["result"]["isError"], json!(true), "{response:?}");
}

#[tokio::test]
async fn missing_required_arg_is_invalid_params() {
    let mut client = start_server("mcp-roundtrip-missingarg", mutations_on()).await;
    let response = client.call_tool("checkout", json!({})).await;
    assert_eq!(response["error"]["code"], json!(-32602), "{response:?}");
}

#[tokio::test]
async fn calling_an_unknown_tool_is_invalid_params() {
    let mut client = start_server("mcp-roundtrip-unknowntool", McpConfig::default()).await;
    let response = client.call_tool("does_not_exist", json!({})).await;
    assert_eq!(response["error"]["code"], json!(-32602), "{response:?}");
}

#[tokio::test]
async fn reset_is_refused_when_mutations_are_disallowed() {
    let repo = FixtureRepoBuilder::new()
        .unwrap()
        .commit("first")
        .unwrap()
        .build();
    let cfg = McpConfig {
        allow_mutations: false,
        ..McpConfig::default()
    };
    let mut client = start_server("mcp-roundtrip-refused", cfg).await;

    let response = client
        .call_tool(
            "reset",
            json!({ "repo": repo.root().to_string_lossy(), "target": "HEAD", "mode": "soft" }),
        )
        .await;
    assert_eq!(response["result"]["isError"], json!(true), "{response:?}");
    assert!(
        response["result"]["structuredContent"]["error"]
            .as_str()
            .unwrap_or_default()
            .contains("allow_mutations"),
        "{response:?}"
    );

    // Reads still run under the same config.
    let status = client
        .call_tool("status", json!({ "repo": repo.root().to_string_lossy() }))
        .await;
    assert_ne!(
        status["result"].get("isError"),
        Some(&json!(true)),
        "{status:?}"
    );
}

// ---- ui_* tools: an agent showing things to the user ----

mod ui {
    use super::*;
    use gitpoulpi_core::domain::{OperationKind, UiControl};
    use gitpoulpi_ipc::transport::{read_message, write_message};
    use gitpoulpi_ipc::ui::{
        UiIntent, UiOrigin, UiOutcome, UiReportArgs, UiReportBody, UiState, ViewerId, ViewerInfo,
    };
    use gitpoulpi_ipc::{Client, Event, Request, ServerMessage};
    use std::sync::{Arc, Mutex};

    type Seen = Arc<Mutex<Vec<(UiIntent, UiOrigin)>>>;

    /// A TUI stand-in attached to the session's published daemon; answers
    /// every intent `Applied` and records what it received.
    async fn attach_fake_viewer(session: &str, repo: &gitpoulpi_testkit::FixtureRepo) -> Seen {
        let port_file =
            gitpoulpi_config::ConfigPaths::for_dir(isolated_config_dir()).ipc_port_file(session);
        let endpoint = gitpoulpi_ipc::discovery::endpoint(&port_file)
            .expect("the MCP server publishes its daemon's port");
        let mut stream = Client::connect(&endpoint).await.unwrap().into_stream();
        let id = repo.repo_ref().id;
        let info = ViewerInfo {
            id: ViewerId("fake".into()),
            pid: 1,
            active_repo: Some(id.clone()),
            tabs: vec![id],
            focused: Some(true),
            agent_control: true,
            last_focus_at: None,
        };
        let req = |n: u64, command, args: Value| Request {
            id: n,
            session: "t".into(),
            repo: None,
            command,
            args,
        };
        write_message(
            &mut stream,
            &req(
                1,
                OperationKind::UiAttach,
                serde_json::to_value(info).unwrap(),
            ),
        )
        .await
        .unwrap();
        // Wait for the attach to be acknowledged before returning.
        loop {
            if let Some(ServerMessage::Response(_)) = read_message(&mut stream).await.unwrap() {
                break;
            }
        }
        let seen: Seen = Arc::default();
        let recorded = Arc::clone(&seen);
        tokio::spawn(async move {
            let mut n = 2;
            while let Ok(Some(message)) = read_message::<_, ServerMessage>(&mut stream).await {
                let ServerMessage::Event(event) = message else {
                    continue;
                };
                let (id, body) = match event {
                    Event::UiIntent {
                        intent_id,
                        intent,
                        origin,
                        ..
                    } => {
                        recorded.lock().unwrap().push((intent, origin));
                        let state = UiState {
                            central: Some("log".into()),
                            ..UiState::default()
                        };
                        (
                            intent_id,
                            UiReportBody::Outcome {
                                outcome: UiOutcome::Applied { state },
                            },
                        )
                    }
                    Event::UiStateRequest { query_id, .. } => (
                        query_id,
                        UiReportBody::State {
                            state: UiState {
                                focus: Some("log".into()),
                                ..UiState::default()
                            },
                        },
                    ),
                    _ => continue,
                };
                let args = serde_json::to_value(UiReportArgs { id, body }).unwrap();
                if write_message(&mut stream, &req(n, OperationKind::UiReport, args))
                    .await
                    .is_err()
                {
                    return;
                }
                n += 1;
            }
        });
        seen
    }

    fn fixture() -> gitpoulpi_testkit::FixtureRepo {
        FixtureRepoBuilder::new()
            .unwrap()
            .commit("first")
            .unwrap()
            .build()
    }

    /// Every `ui_*` tool is listed, annotated as a non-mutating,
    /// non-destructive, closed-world read (an agent client may then run it
    /// without a prompt), and the daemon-lifecycle tools are still absent.
    #[tokio::test]
    async fn ui_tools_are_listed_as_view_only() {
        let mut client = start_server("mcp-ui-list", McpConfig::default()).await;
        let response = client.request("tools/list", json!({})).await;
        let tools = response["result"]["tools"].as_array().unwrap();
        let ui: Vec<&Value> = tools
            .iter()
            .filter(|t| t["name"].as_str().unwrap().starts_with("ui_"))
            .collect();
        let names: Vec<&str> = ui.iter().map(|t| t["name"].as_str().unwrap()).collect();
        for expected in [
            "ui_viewers",
            "ui_state",
            "ui_show_tab",
            "ui_show_home",
            "ui_show_commit",
            "ui_show_search",
            "ui_show_ref",
            "ui_show_pr",
            "ui_show_issue",
            "ui_show_wip",
            "ui_show_diff",
            "ui_show_file",
            "ui_show_conflict",
            "ui_propose_resolution",
            "ui_show_settings",
            "ui_show_palette",
            "ui_show_keys",
            "ui_highlight",
            "ui_notify",
            "ui_back",
        ] {
            assert!(names.contains(&expected), "{expected} not in {names:?}");
        }
        assert_eq!(ui.len(), 26, "{names:?}");
        for tool in ui {
            let a = &tool["annotations"];
            if tool["name"].as_str().unwrap().starts_with("ui_propose_") {
                // Their own class: a change the user confirms.
                assert_eq!(a["readOnlyHint"], json!(false), "{}", tool["name"]);
                assert_eq!(a["destructiveHint"], json!(false), "{}", tool["name"]);
                assert_eq!(a["idempotentHint"], json!(false), "{}", tool["name"]);
                continue;
            }
            assert_eq!(a["readOnlyHint"], json!(true), "{}", tool["name"]);
            assert_eq!(a["destructiveHint"], json!(false), "{}", tool["name"]);
            assert_eq!(a["idempotentHint"], json!(true), "{}", tool["name"]);
            assert_eq!(a["openWorldHint"], json!(false), "{}", tool["name"]);
        }
        let settings = tools
            .iter()
            .find(|t| t["name"] == "ui_show_settings")
            .unwrap();
        let pages = settings["inputSchema"]["properties"]["page"]["enum"]
            .as_array()
            .expect("the page ids are an enum in the schema");
        assert!(pages.contains(&json!("diff-editor")));
        assert!(
            pages.contains(&json!("ui-customization")),
            "legacy ids stay accepted"
        );
        assert!(pages.contains(&json!("open-in")));
    }

    #[tokio::test]
    async fn a_show_call_reaches_the_window_with_the_operators_mode_and_the_agents_name() {
        let repo = fixture();
        let cfg = McpConfig {
            ui_control: UiControl::Auto,
            // The mutation gate does not apply to view tools.
            allow_mutations: false,
            ..McpConfig::default()
        };
        let mut client = start_server("mcp-ui-show", cfg).await;
        let seen = attach_fake_viewer("mcp-ui-show", &repo).await;
        let root = repo.root().to_string_lossy().to_string();

        let shown = client
            .call_tool("ui_show_commit", json!({ "rev": "HEAD", "repo": root }))
            .await;
        assert_ne!(shown["result"]["isError"], json!(true), "{shown}");
        assert_eq!(shown["result"]["structuredContent"]["outcome"], "applied");
        let seen = seen.lock().unwrap().clone();
        assert!(matches!(seen[0].0, UiIntent::ShowCommit { .. }));
        assert_eq!(seen[0].1.client, "gitpoulpi-mcp-tests");
        assert_eq!(seen[0].1.control, UiControl::Auto);

        // `ui_state` reads the window back.
        let state = client.call_tool("ui_state", json!({ "repo": root })).await;
        assert_eq!(
            state["result"]["structuredContent"]["focus"], "log",
            "{state}"
        );

        // A bad revision never reaches the window.
        let bad = client
            .call_tool("ui_show_commit", json!({ "rev": "nope", "repo": root }))
            .await;
        assert_eq!(bad["result"]["isError"], json!(true));
        assert_eq!(bad["result"]["structuredContent"]["outcome"], "not_found");
    }

    #[tokio::test]
    async fn without_a_window_the_tool_errors_with_a_clear_message() {
        let repo = fixture();
        let mut client = start_server("mcp-ui-noviewer", McpConfig::default()).await;
        let response = client
            .call_tool(
                "ui_show_commit",
                json!({ "rev": "HEAD", "repo": repo.root().to_string_lossy() }),
            )
            .await;
        assert_eq!(response["result"]["isError"], json!(true), "{response}");
        let out = &response["result"]["structuredContent"];
        assert_eq!(out["outcome"], "no_viewer");
        assert!(out["message"]
            .as_str()
            .unwrap()
            .contains("No GitPoulpi window"));
    }

    #[tokio::test]
    async fn allow_ui_false_refuses_show_tools_but_not_state_and_viewers() {
        let repo = fixture();
        let cfg = McpConfig {
            allow_ui: false,
            ..McpConfig::default()
        };
        let mut client = start_server("mcp-ui-gated", cfg).await;
        let seen = attach_fake_viewer("mcp-ui-gated", &repo).await;
        let root = repo.root().to_string_lossy().to_string();

        for (tool, args) in [
            ("ui_show_commit", json!({ "rev": "HEAD", "repo": root })),
            ("ui_notify", json!({ "message": "hi", "repo": root })),
            ("ui_back", json!({ "repo": root })),
        ] {
            let r = client.call_tool(tool, args).await;
            assert_eq!(r["result"]["isError"], json!(true), "{tool}: {r}");
            assert!(
                r["result"]["structuredContent"]["error"]
                    .as_str()
                    .unwrap()
                    .contains("allow_ui"),
                "{tool}: {r}"
            );
        }
        assert!(
            seen.lock().unwrap().is_empty(),
            "nothing reached the window"
        );

        let viewers = client
            .call_tool("ui_viewers", json!({ "repo": root }))
            .await;
        assert_ne!(viewers["result"]["isError"], json!(true), "{viewers}");
        assert_eq!(viewers["result"]["structuredContent"][0]["id"], "fake");
        let state = client.call_tool("ui_state", json!({ "repo": root })).await;
        assert_ne!(state["result"]["isError"], json!(true), "{state}");
    }

    /// `ui_propose_rewrite` opens the workbench on the user's side and does
    /// nothing else: the viewer gets one intent, a script is refused before
    /// it, and the repository is as it was.
    #[tokio::test]
    #[cfg(feature = "history-rewrite")]
    async fn a_rewrite_proposal_reaches_the_window_and_rewrites_nothing() {
        use gitpoulpi_core::domain::rewrite::{
            Recipe, Rule, RuleKind, Scope, ScriptLanguage, ScriptRef, ScriptSource,
        };
        let repo = fixture();
        let cfg = McpConfig {
            allow_ui_propose: true,
            allow_mutations: false,
            ..McpConfig::default()
        };
        let mut client = start_server("mcp-ui-propose-rewrite", cfg).await;
        let seen = attach_fake_viewer("mcp-ui-propose-rewrite", &repo).await;
        let root = repo.root().to_string_lossy().to_string();
        let head = repo.git(&["rev-parse", "HEAD"]);
        let dir = tempfile::tempdir().unwrap();
        let file = dir.path().join("recipe.json");
        let write = |recipe: &Recipe| {
            std::fs::write(&file, serde_json::to_string(recipe).unwrap()).unwrap();
            file.to_string_lossy().to_string()
        };

        let script = Recipe::new(
            "script",
            Scope::default(),
            vec![Rule::new(RuleKind::Script(ScriptRef {
                language: ScriptLanguage::Lua,
                source: ScriptSource::Inline { text: "x".into() },
                params: Default::default(),
            }))],
        );
        let refused = client
            .call_tool(
                "ui_propose_rewrite",
                json!({ "recipe": write(&script), "repo": root }),
            )
            .await;
        assert_eq!(refused["result"]["isError"], json!(true), "{refused}");
        assert!(seen.lock().unwrap().is_empty(), "a script never reaches it");

        let recipe = Recipe::new(
            "fix",
            Scope::default(),
            vec![Rule::new(RuleKind::MessageEncoding { names: false })],
        );
        let sent = client
            .call_tool(
                "ui_propose_rewrite",
                json!({ "recipe": write(&recipe), "repo": root }),
            )
            .await;
        assert_ne!(sent["result"]["isError"], json!(true), "{sent}");
        {
            let seen = seen.lock().unwrap();
            assert_eq!(seen.len(), 1);
            let UiIntent::Propose {
                proposal: gitpoulpi_ipc::ui::Proposal::Rewrite { recipe },
                ..
            } = &seen[0].0
            else {
                panic!("{:?}", seen[0])
            };
            assert_eq!(recipe.rules.len(), 1);
        }
        assert_eq!(repo.git(&["rev-parse", "HEAD"]), head);
        assert!(
            !repo
                .root()
                .join(".git")
                .join("gitpoulpi")
                .join("rewrite")
                .exists(),
            "no rewrite run was started"
        );
    }

    #[tokio::test]
    async fn ui_control_off_refuses_show_tools() {
        let repo = fixture();
        let cfg = McpConfig {
            ui_control: UiControl::Off,
            ..McpConfig::default()
        };
        let mut client = start_server("mcp-ui-off", cfg).await;
        let r = client
            .call_tool(
                "ui_notify",
                json!({ "message": "hi", "repo": repo.root().to_string_lossy() }),
            )
            .await;
        assert_eq!(r["result"]["isError"], json!(true));
        assert!(r["result"]["structuredContent"]["error"]
            .as_str()
            .unwrap()
            .contains("ui control is off"));
    }
}

#[cfg(feature = "full")]
#[tokio::test]
async fn the_purge_tool_does_not_exist_unless_the_operator_opts_in() {
    let names = |response: &Value| -> Vec<String> {
        response["result"]["tools"]
            .as_array()
            .expect("tools/list must return an array")
            .iter()
            .map(|t| t["name"].as_str().unwrap_or_default().to_string())
            .collect()
    };

    // Default: the rewrite tools are there, the purge is not, even with
    // mutations on; calling it by name says which key to set.
    let mut client = start_server("mcp-roundtrip-purge-off", mutations_on()).await;
    let listed = names(&client.request("tools/list", json!({})).await);
    assert!(listed.iter().any(|n| n == "rewrite_push"), "{listed:?}");
    assert!(!listed.iter().any(|n| n == "rewrite_purge"), "{listed:?}");
    let response = client.call_tool("rewrite_purge", json!({})).await;
    assert_eq!(response["result"]["isError"], json!(true), "{response:?}");
    assert!(
        response["result"]["structuredContent"]["error"]
            .as_str()
            .unwrap_or_default()
            .contains("allow_history_purge"),
        "{response:?}"
    );

    // Opted in: listed, and annotated as destructive.
    let cfg = McpConfig {
        allow_mutations: true,
        allow_history_purge: true,
        ..McpConfig::default()
    };
    let mut client = start_server("mcp-roundtrip-purge-on", cfg).await;
    let response = client.request("tools/list", json!({})).await;
    let tool = response["result"]["tools"]
        .as_array()
        .unwrap()
        .iter()
        .find(|t| t["name"] == "rewrite_purge")
        .expect("`rewrite_purge` is listed once the operator opts in");
    assert_eq!(tool["annotations"]["destructiveHint"], json!(true));
    assert_eq!(tool["annotations"]["readOnlyHint"], json!(false));

    // Opted in but mutations off: still refused.
    let cfg = McpConfig {
        allow_history_purge: true,
        ..McpConfig::default()
    };
    let mut client = start_server("mcp-roundtrip-purge-readonly", cfg).await;
    let response = client.call_tool("rewrite_purge", json!({})).await;
    assert_eq!(response["result"]["isError"], json!(true), "{response:?}");
    assert!(
        response["result"]["structuredContent"]["error"]
            .as_str()
            .unwrap_or_default()
            .contains("allow_mutations"),
        "{response:?}"
    );
}

// No test reads or writes the real config, data dir or credentials of whoever runs it.
gitpoulpi_testkit::isolate_binary!();
