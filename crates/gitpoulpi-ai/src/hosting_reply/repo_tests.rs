//! The repository part of a drafted reply: planning, building, privacy,
//! budget, caches, the two routes' set-up, the Anthropic cache blocks.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use gitpoulpi_config::{AiAction, AiConfig, AiLimits, AiProfile, AiProviderKind};
use gitpoulpi_core::domain::{GrepHit, Oid};
use gitpoulpi_secrets::InMemoryTokens;

use super::cache::{ContextCache, SessionKey, SessionStore};
use super::plan::{plan_context, ContextSelection, FileRef};
use super::prompt::tests::{comment, request};
use super::repo_access::{read_repository, settle, RepoSetup};
use super::repo_context::{build, BuildInput, ContextLimits, PartKind, RepoReader};
use super::*;
use crate::privacy::Privacy;

const TOKEN: &str = "ghp_q7Xk2LmZp9RtV4bN8wYcJ3hGfD5sAe6U1oIx";

fn block_on<T>(f: impl std::future::Future<Output = T>) -> T {
    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(f)
}

fn oid(n: u8) -> Oid {
    Oid([n; 20])
}

fn limits() -> ContextLimits {
    ContextLimits::from_config(&AiLimits::default())
}

/// A repository in memory that counts what it is asked.
struct Fake {
    head: Mutex<Oid>,
    tree: Vec<String>,
    files: HashMap<String, Vec<u8>>,
    hits: Vec<GrepHit>,
    diff: Option<String>,
    reads: Mutex<Vec<String>>,
    greps: Mutex<usize>,
}

impl Fake {
    fn new() -> Self {
        let mut files = HashMap::new();
        files.insert("src/a.rs".to_string(), b"fn a() {}\n".to_vec());
        files.insert("src/deep/b.rs".to_string(), b"fn b() {}\n".to_vec());
        files.insert(
            "src/secret.rs".to_string(),
            format!("const KEY: &str = \"{TOKEN}\";\nfn s() {{}}\n").into_bytes(),
        );
        files.insert(".env".to_string(), b"DB_PASSWORD=hunter2hunter2\n".to_vec());
        Self {
            head: Mutex::new(oid(1)),
            tree: files.keys().cloned().collect(),
            files,
            hits: Vec::new(),
            diff: None,
            reads: Mutex::new(Vec::new()),
            greps: Mutex::new(0),
        }
    }
}

#[async_trait::async_trait]
impl RepoReader for Fake {
    async fn head(&self) -> Result<Oid> {
        Ok(*self.head.lock().unwrap())
    }
    async fn list_tree(&self, _at: Oid) -> Result<Vec<String>> {
        let mut tree = self.tree.clone();
        tree.sort();
        Ok(tree)
    }
    async fn read(&self, path: &str, _at: Oid) -> Result<Option<Vec<u8>>> {
        self.reads.lock().unwrap().push(path.to_string());
        Ok(self.files.get(path).cloned())
    }
    async fn grep(&self, terms: &[String], _at: Oid, _max: usize) -> Result<Vec<GrepHit>> {
        *self.greps.lock().unwrap() += 1;
        Ok(self
            .hits
            .iter()
            .filter(|h| terms.contains(&h.term))
            .cloned()
            .collect())
    }
    async fn diff(&self, _base: &str, _head: &str) -> Result<Option<String>> {
        Ok(self.diff.clone())
    }
}

fn selection(files: &[&str]) -> ContextSelection {
    ContextSelection {
        files: files
            .iter()
            .map(|p| FileRef {
                path: (*p).to_string(),
                line: None,
            })
            .collect(),
        repo_map: true,
        ..ContextSelection::default()
    }
}

fn built(reader: &Fake, selection: &ContextSelection, limits: ContextLimits) -> RepoContext {
    let mut cache = ContextCache::new(30);
    built_with(reader, selection, limits, &mut cache)
}

fn built_with(
    reader: &Fake,
    selection: &ContextSelection,
    limits: ContextLimits,
    cache: &mut ContextCache,
) -> RepoContext {
    let tree = block_on(reader.list_tree(oid(1))).unwrap();
    let privacy = Privacy::default();
    block_on(build(
        &BuildInput {
            reader,
            root: std::path::Path::new("/repo"),
            head: oid(1),
            tree: &tree,
            selection,
            limits,
            privacy: &privacy,
            now: cache.now(),
        },
        cache.parts(),
    ))
    .unwrap()
}

// -- plan_context -----------------------------------------------------------------

fn tree() -> Vec<String> {
    ["README.md", "docs/x.md", "src/a.rs", "src/deep/b.rs"]
        .map(String::from)
        .to_vec()
}

#[test]
fn the_plan_picks_paths_frames_backticks_quotes_and_title_words() {
    let mut r = request(vec![comment(
        "c1",
        "bob",
        "It dies at f (src/deep/b.rs:30) and the message is \"unexpected end of input\".",
    )]);
    r.item = Some(ItemDigest {
        summary: "Issue #3 \"Crash when parsing dates\" (open) by al".into(),
        description: "See src/a.rs:12 and `parse_date()`; also b.rs#L7 and no/such.rs.".into(),
    });
    r.diff = Some(DiffRange {
        base: "main".into(),
        head: "topic".into(),
    });
    let plan = plan_context(&r, &tree(), &limits());
    let files: Vec<(&str, Option<u32>)> = plan
        .files
        .iter()
        .map(|f| (f.path.as_str(), f.line))
        .collect();
    assert_eq!(
        files,
        [("src/a.rs", Some(12)), ("src/deep/b.rs", Some(7))],
        "the planned files exist in the tree, once each (the first mention's line); \
         no/such.rs is dropped"
    );
    assert_eq!(plan.symbols, ["parse_date"]);
    assert!(plan.terms.contains(&"unexpected end of input".to_string()));
    assert!(
        plan.terms.contains(&"parsing".to_string()),
        "{:?}",
        plan.terms
    );
    assert_eq!(plan.diff.as_ref().unwrap().base, "main");
    assert!(plan.repo_map);
}

#[test]
fn the_plan_holds_the_number_of_terms_to_the_limit() {
    let mut r = request(Vec::new());
    r.item = Some(ItemDigest {
        summary: "Issue #3 \"Alphabetical headers regression\" (open) by al".into(),
        description: "`one_name` `two_name` `three_name` `four_name`".into(),
    });
    let mut small = limits();
    small.terms = 3;
    let plan = plan_context(&r, &tree(), &small);
    assert!(plan.symbols.len() + plan.terms.len() <= 3, "{plan:?}");
    assert_eq!(plan.symbols.len(), 3, "symbols come first");
}

#[test]
fn the_selection_hash_changes_with_the_selection() {
    assert_ne!(
        selection(&["src/a.rs"]).hash(),
        selection(&["src/deep/b.rs"]).hash()
    );
    assert_eq!(
        selection(&["src/a.rs"]).hash(),
        selection(&["src/a.rs"]).hash()
    );
}

// -- the builder: privacy and budget ----------------------------------------------

#[test]
fn a_secret_in_a_file_is_masked_and_a_sensitive_file_is_never_read() {
    let reader = Fake::new();
    let ctx = built(&reader, &selection(&["src/secret.rs", ".env"]), limits());
    let text = ctx.block().text;
    assert!(!text.contains(TOKEN), "the token leaked: {text}");
    assert!(text.contains("[REDACTED:"), "{text}");
    assert!(!text.contains("hunter2"), "{text}");
    assert_eq!(ctx.left_out, [".env"]);
    assert!(
        !reader.reads.lock().unwrap().contains(&".env".to_string()),
        "a sensitive file is not even read"
    );
    assert!(ctx.redactions >= 1);
}

#[test]
fn a_secret_in_the_diff_and_in_search_hits_is_masked_and_a_sensitive_diff_file_dropped() {
    let mut reader = Fake::new();
    reader.diff = Some(format!(
        "diff -- src/x.rs (modified)\n@@ -1,1 +1,2 @@\n a\n+let k = \"{TOKEN}\";\n\
         diff -- config/.env (new file)\n@@ -0,0 +1,1 @@\n+DB_PASSWORD=hunter2hunter2\n"
    ));
    reader.hits = vec![GrepHit {
        term: "KEY".into(),
        path: "src/secret.rs".into(),
        line: 1,
        text: format!("const KEY: &str = \"{TOKEN}\";"),
    }];
    let mut plan = selection(&[]);
    plan.diff = Some(DiffRange {
        base: "a".into(),
        head: "b".into(),
    });
    plan.terms = vec!["KEY".into()];
    let ctx = built(&reader, &plan, limits());
    let text = ctx.block().text;
    assert!(!text.contains(TOKEN) && !text.contains("hunter2"), "{text}");
    assert!(text.contains("src/x.rs") && !text.contains("config/.env"));
    assert!(ctx
        .parts
        .iter()
        .any(|p| matches!(p.kind, PartKind::SearchHits { .. })));
    assert_eq!(ctx.left_out, ["config/.env"]);
}

#[test]
fn over_the_budget_the_cuts_follow_the_priority_and_are_reported() {
    let mut reader = Fake::new();
    reader.files.insert(
        "src/a.rs".into(),
        format!("// {}\n", "a".repeat(900)).into_bytes(),
    );
    reader.diff = Some(format!(
        "diff -- src/x.rs (modified)\n+{}\n",
        "d".repeat(900)
    ));
    let mut plan = selection(&["src/a.rs"]);
    plan.diff = Some(DiffRange {
        base: "a".into(),
        head: "b".into(),
    });
    let mut small = limits();
    small.total = 1_200;
    let ctx = built(&reader, &plan, small);
    assert!(ctx.total <= 1_200, "{}", ctx.total);
    let kinds: Vec<&PartKind> = ctx.parts.iter().map(|p| &p.kind).collect();
    assert!(kinds.contains(&&PartKind::Diff), "the diff is kept first");
    assert!(!ctx.cuts.is_empty(), "the budget cut is reported");
    assert!(
        ctx.cuts.iter().all(|c| c.what != "Pull request diff"),
        "the diff, first in priority, is not the one cut: {:?}",
        ctx.cuts
    );
}

#[test]
fn a_window_of_lines_is_read_around_a_referenced_line_of_a_big_file() {
    let mut reader = Fake::new();
    let big: String = (1..=500).map(|i| format!("line {i}\n")).collect();
    reader.files.insert("src/a.rs".into(), big.into_bytes());
    let mut plan = selection(&[]);
    plan.files = vec![FileRef {
        path: "src/a.rs".into(),
        line: Some(250),
    }];
    let mut small = limits();
    small.file = 2_000;
    let ctx = built(&reader, &plan, small);
    let file = ctx
        .parts
        .iter()
        .find(|p| matches!(p.kind, PartKind::File { .. }))
        .unwrap();
    assert!(file.text.contains("line 250") && !file.text.contains("line 1\n"));
    assert!(matches!(
        file.kind,
        PartKind::File {
            lines: Some((210, 290)),
            ..
        }
    ));
}

// -- the caches --------------------------------------------------------------------

fn http_job() -> Job {
    let mut config = AiConfig {
        enabled: true,
        ..AiConfig::default()
    };
    config
        .profiles
        .push(AiProfile::new("p", "P", AiProviderKind::Anthropic));
    config.default_profile = "p".into();
    Job::prepare(
        &config,
        AiAction::HostingReply,
        None,
        &InMemoryTokens::default(),
    )
    .unwrap()
}

fn agent_job(args: &[&str]) -> Job {
    let mut config = AiConfig {
        enabled: true,
        ..AiConfig::default()
    };
    let mut profile = AiProfile::new("p", "P", AiProviderKind::LocalAgent);
    profile.command = "claude".into();
    profile.args = args.iter().map(|a| (*a).to_string()).collect();
    config.profiles.push(profile);
    config.default_profile = "p".into();
    Job::prepare(
        &config,
        AiAction::HostingReply,
        None,
        &InMemoryTokens::default(),
    )
    .unwrap()
}

fn setup<'a>(
    reader: &'a Fake,
    cache: &'a mut ContextCache,
    sessions: &'a mut SessionStore,
) -> RepoSetup<'a> {
    RepoSetup {
        reader,
        root: std::path::Path::new("/repo"),
        cache,
        sessions,
        reuse_session: true,
        refresh: false,
        limits: limits(),
        privacy: Privacy::default(),
    }
}

fn plain_request(text: &str) -> ReplyRequest {
    request(vec![comment("a", "bob", text)])
}

fn context_of(r: &ReplyRequest) -> Arc<RepoContext> {
    match &r.repo {
        RepoAccess::Built(c) => c.clone(),
        other => panic!("not a built context: {other:?}"),
    }
}

#[test]
fn the_same_reply_twice_reuses_the_context_and_reads_nothing_again() {
    let reader = Fake::new();
    let mut cache = ContextCache::new(30);
    let mut sessions = SessionStore::new();
    let mut first = plain_request("look at src/a.rs please");
    block_on(read_repository(
        http_job(),
        &mut first,
        &mut setup(&reader, &mut cache, &mut sessions),
    ))
    .unwrap();
    assert_eq!(context_of(&first).reused_parts(), 0);
    let reads = reader.reads.lock().unwrap().len();
    let mut second = plain_request("look at src/a.rs please");
    block_on(read_repository(
        http_job(),
        &mut second,
        &mut setup(&reader, &mut cache, &mut sessions),
    ))
    .unwrap();
    let again = context_of(&second);
    assert_eq!(again.reused_parts(), again.parts.len(), "all parts reused");
    assert_eq!(
        reader.reads.lock().unwrap().len(),
        reads,
        "nothing read again"
    );
}

#[test]
fn a_consecutive_issue_in_the_same_repository_rebuilds_only_its_new_parts() {
    let reader = Fake::new();
    let mut cache = ContextCache::new(30);
    let mut sessions = SessionStore::new();
    let mut one = plain_request("see src/a.rs");
    block_on(read_repository(
        http_job(),
        &mut one,
        &mut setup(&reader, &mut cache, &mut sessions),
    ))
    .unwrap();
    let before = reader.reads.lock().unwrap().len();
    let mut two = plain_request("see src/a.rs and src/deep/b.rs");
    block_on(read_repository(
        http_job(),
        &mut two,
        &mut setup(&reader, &mut cache, &mut sessions),
    ))
    .unwrap();
    let ctx = context_of(&two);
    let reused: Vec<String> = ctx
        .parts
        .iter()
        .filter(|p| p.reused)
        .map(|p| p.kind.label())
        .collect();
    let fresh: Vec<String> = ctx
        .parts
        .iter()
        .filter(|p| !p.reused)
        .map(|p| p.kind.label())
        .collect();
    assert!(reused.contains(&"src/a.rs".to_string()), "{reused:?}");
    assert!(fresh.contains(&"src/deep/b.rs".to_string()), "{fresh:?}");
    assert_eq!(
        reader.reads.lock().unwrap().len(),
        before + 1,
        "only the new file is read"
    );
}

#[test]
fn turning_redaction_on_does_not_find_the_context_cached_with_it_off() {
    let reader = Fake::new();
    let mut cache = ContextCache::new(30);
    let mut sessions = SessionStore::new();
    let ask =
        |reader: &Fake, cache: &mut ContextCache, sessions: &mut SessionStore, redact: bool| {
            let mut r = plain_request("see src/secret.rs");
            let mut s = setup(reader, cache, sessions);
            s.privacy.redact_secrets = redact;
            block_on(read_repository(http_job(), &mut r, &mut s)).unwrap();
            context_of(&r)
        };
    let open = ask(&reader, &mut cache, &mut sessions, false);
    assert!(open.block().text.contains(TOKEN), "redaction was off");
    let again = ask(&reader, &mut cache, &mut sessions, false);
    assert_eq!(again.reused_parts(), again.parts.len(), "same settings hit");
    let masked = ask(&reader, &mut cache, &mut sessions, true);
    let text = masked.block().text;
    assert!(
        !text.contains(TOKEN),
        "the cached clear text leaked: {text}"
    );
    assert_eq!(masked.reused_parts(), 0, "rebuilt under the new settings");
}

#[test]
fn a_new_head_and_refresh_both_rebuild() {
    let reader = Fake::new();
    let mut cache = ContextCache::new(30);
    let mut sessions = SessionStore::new();
    let ask = |reader: &Fake, cache: &mut ContextCache, sessions: &mut SessionStore, refresh| {
        let mut r = plain_request("see src/a.rs");
        let mut s = setup(reader, cache, sessions);
        s.refresh = refresh;
        block_on(read_repository(http_job(), &mut r, &mut s)).unwrap();
        context_of(&r)
    };
    let first = ask(&reader, &mut cache, &mut sessions, false);
    assert_eq!(first.reused_parts(), 0);
    let hit = ask(&reader, &mut cache, &mut sessions, false);
    assert_eq!(hit.reused_parts(), hit.parts.len());
    // HEAD moves.
    *reader.head.lock().unwrap() = oid(2);
    let moved = ask(&reader, &mut cache, &mut sessions, false);
    assert_eq!(moved.reused_parts(), 0, "a new HEAD rebuilds");
    assert_eq!(moved.key.head, oid(2));
    assert!(
        cache.get(&first.key).is_none(),
        "the old commit's context is gone"
    );
    // Refresh forces a rebuild at the same HEAD.
    let kept = ask(&reader, &mut cache, &mut sessions, false);
    assert_eq!(kept.reused_parts(), kept.parts.len());
    let refreshed = ask(&reader, &mut cache, &mut sessions, true);
    assert_eq!(refreshed.reused_parts(), 0, "Refresh context rebuilds");
}

#[test]
fn a_context_older_than_the_ttl_is_not_reused() {
    use std::sync::atomic::{AtomicU64, Ordering};
    static NOW: AtomicU64 = AtomicU64::new(1_000);
    fn clock() -> u64 {
        NOW.load(Ordering::SeqCst)
    }
    let reader = Fake::new();
    let mut cache = ContextCache::new(1).with_clock(clock);
    let mut sessions = SessionStore::new();
    let ask = |cache: &mut ContextCache, sessions: &mut SessionStore| {
        let mut r = plain_request("see src/a.rs");
        block_on(read_repository(
            http_job(),
            &mut r,
            &mut setup(&reader, cache, sessions),
        ))
        .unwrap();
        context_of(&r)
    };
    ask(&mut cache, &mut sessions);
    NOW.store(1_000 + 30, Ordering::SeqCst);
    let young = ask(&mut cache, &mut sessions);
    assert_eq!(
        young.reused_parts(),
        young.parts.len(),
        "30 s is within 1 minute"
    );
    NOW.store(1_000 + 61, Ordering::SeqCst);
    let old = ask(&mut cache, &mut sessions);
    assert_eq!(
        old.reused_parts(),
        0,
        "past the ttl everything is read again"
    );
}

#[test]
fn the_disk_cache_survives_a_restart_and_is_user_only() {
    let dir = tempfile::tempdir().unwrap();
    let reader = Fake::new();
    let root = std::path::Path::new("/repo");
    let mut cache = ContextCache::on_disk(dir.path(), root, 30);
    built_with(&reader, &selection(&["src/a.rs"]), limits(), &mut cache);
    let ctx = built_with(&reader, &selection(&["src/a.rs"]), limits(), &mut cache);
    cache.put(ctx.clone());
    drop(cache);
    let mut again = ContextCache::on_disk(dir.path(), root, 30);
    assert!(
        again.get(&ctx.key).is_some(),
        "the built context is read back"
    );
    assert!(!again.parts().is_empty(), "so are the parts");
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        for entry in std::fs::read_dir(dir.path()).unwrap() {
            let mode = entry.unwrap().metadata().unwrap().permissions().mode();
            assert_eq!(mode & 0o077, 0, "the cache is private to the user");
        }
    }
}

// -- the agent route: read-only set-up and sessions ---------------------------------

fn flag_value(args: &[String], flag: &str) -> Option<String> {
    args.iter()
        .position(|a| a == flag)
        .and_then(|i| args.get(i + 1))
        .cloned()
}

#[test]
fn an_agent_reads_with_read_only_tools_in_the_repository_without_credentials() {
    let reader = Fake::new();
    let mut cache = ContextCache::new(30);
    let mut sessions = SessionStore::new();
    let mut r = plain_request("hi");
    let (job, _) = block_on(read_repository(
        agent_job(&["-p", "--tools", "", "--no-session-persistence"]),
        &mut r,
        &mut setup(&reader, &mut cache, &mut sessions),
    ))
    .unwrap();
    let args = &job.profile.args;
    assert_eq!(
        flag_value(args, "--tools").as_deref(),
        Some("Read,Grep,Glob")
    );
    let denied = flag_value(args, "--disallowedTools").unwrap();
    for tool in ["Bash", "Edit", "Write", "WebFetch", "NotebookEdit"] {
        assert!(denied.contains(tool), "{tool} must be refused: {args:?}");
    }
    assert!(args.contains(&"--strict-mcp-config".to_string()));
    assert_eq!(
        flag_value(args, "--mcp-config").as_deref(),
        Some("{\"mcpServers\":{}}")
    );
    assert_eq!(args.iter().filter(|a| *a == "--tools").count(), 1);
    assert!(
        job.runs_in_repository(),
        "the agent works in the repository"
    );
    assert!(job.scrub_env, "without the hosting credentials");
    assert!(matches!(r.repo, RepoAccess::AgentReadOnly { .. }));
    assert!(
        job.system_prompt().len() > 10 && !reader.reads.lock().unwrap().iter().any(|_| true),
        "an agent route builds no context and reads nothing itself"
    );
}

#[test]
fn an_agent_without_a_trusted_read_only_mode_is_refused() {
    let reader = Fake::new();
    let mut cache = ContextCache::new(30);
    let mut sessions = SessionStore::new();
    let mut config = AiConfig {
        enabled: true,
        ..AiConfig::default()
    };
    let mut profile = AiProfile::new("p", "P", AiProviderKind::LocalAgent);
    profile.command = "aider".into();
    profile.harness = "aider".into();
    config.profiles.push(profile.clone());
    config.default_profile = "p".into();
    let job = Job::prepare(
        &config,
        AiAction::HostingReply,
        None,
        &InMemoryTokens::default(),
    )
    .unwrap();
    assert_eq!(
        super::repo_access::blocked_reason(&profile),
        Some("This agent has no read-only mode GitPoulpi trusts")
    );
    let mut r = plain_request("hi");
    let result = block_on(read_repository(
        job,
        &mut r,
        &mut setup(&reader, &mut cache, &mut sessions),
    ));
    assert!(matches!(result, Err(Error::Invalid(_))), "{result:?}");
    assert_eq!(r.repo, RepoAccess::Off);
}

#[test]
fn codex_is_set_to_its_read_only_sandbox() {
    let harness = crate::presets::harness_named("codex").unwrap();
    let args: Vec<String> = ["exec", "-", "--sandbox", "workspace-write", "--ephemeral"]
        .map(String::from)
        .to_vec();
    let out = harness.with_read_only_tools(&args).unwrap();
    assert_eq!(flag_value(&out, "--sandbox").as_deref(), Some("read-only"));
    assert_eq!(out.iter().filter(|a| *a == "--sandbox").count(), 1);
    assert!(out.contains(&"--ephemeral".to_string()));
}

#[test]
fn the_session_is_started_then_resumed_and_a_new_head_starts_a_new_one() {
    let reader = Fake::new();
    let mut cache = ContextCache::new(30);
    let mut sessions = SessionStore::new();
    let run = |reader: &Fake, cache: &mut ContextCache, sessions: &mut SessionStore, reuse| {
        let mut r = plain_request("hi");
        let mut s = setup(reader, cache, sessions);
        s.reuse_session = reuse;
        let (job, pending) = block_on(read_repository(
            agent_job(&["-p", "--no-session-persistence"]),
            &mut r,
            &mut s,
        ))
        .unwrap();
        settle(sessions, &pending, true);
        (job.profile.args.clone(), r.repo.clone())
    };
    let (first, note) = run(&reader, &mut cache, &mut sessions, true);
    let id = flag_value(&first, "--session-id").expect("the first run starts a session");
    assert!(!first.contains(&"--no-session-persistence".to_string()));
    assert!(matches!(
        note,
        RepoAccess::AgentReadOnly {
            session: Some(AgentSession::Started),
            ..
        }
    ));
    let (second, note) = run(&reader, &mut cache, &mut sessions, true);
    assert_eq!(
        flag_value(&second, "--resume").as_deref(),
        Some(id.as_str())
    );
    assert!(!second.contains(&"--session-id".to_string()));
    assert!(matches!(
        note,
        RepoAccess::AgentReadOnly {
            session: Some(AgentSession::Continued),
            ..
        }
    ));
    // A commit moves HEAD: a new session.
    *reader.head.lock().unwrap() = oid(2);
    let (third, _) = run(&reader, &mut cache, &mut sessions, true);
    let fresh = flag_value(&third, "--session-id").expect("a new session after a HEAD change");
    assert_ne!(fresh, id);
    // With the setting off the run stays one-shot.
    let (off, note) = run(&reader, &mut cache, &mut sessions, false);
    assert!(off.contains(&"--no-session-persistence".to_string()));
    assert!(!off.contains(&"--resume".to_string()) && !off.contains(&"--session-id".to_string()));
    assert!(matches!(
        note,
        RepoAccess::AgentReadOnly { session: None, .. }
    ));
}

#[test]
fn a_session_whose_first_run_failed_is_forgotten() {
    let mut sessions = SessionStore::new();
    let key = SessionKey {
        repo_root: "/repo".into(),
        head: "abc".into(),
        action: AiAction::HostingReply,
    };
    let first = sessions.next_use(&key);
    sessions.forget_unstarted(&key);
    let second = sessions.next_use(&key);
    assert_ne!(
        first, second,
        "a failed first run leaves no session to resume"
    );
    sessions.mark_started(&key);
    assert!(matches!(
        sessions.next_use(&key),
        crate::presets::SessionUse::Resume(_)
    ));
}

#[test]
fn the_session_file_is_kept_and_read_back() {
    let dir = tempfile::tempdir().unwrap();
    let file = dir.path().join("ai-sessions.toml");
    let key = SessionKey {
        repo_root: "/repo".into(),
        head: "abc".into(),
        action: AiAction::HostingReply,
    };
    let mut sessions = SessionStore::on_disk(&file);
    sessions.next_use(&key);
    sessions.mark_started(&key);
    let mut again = SessionStore::on_disk(&file);
    assert!(matches!(
        again.next_use(&key),
        crate::presets::SessionUse::Resume(_)
    ));
}

#[test]
fn credential_variables_are_the_ones_taken_out_of_the_agents_environment() {
    let names = [
        "GH_TOKEN",
        "github_token",
        "GITLAB_TOKEN",
        "CI_JOB_TOKEN",
        "BITBUCKET_PASSWORD",
        "AZURE_DEVOPS_EXT_PAT",
        "JIRA_API_TOKEN",
        "LINEAR_API_KEY",
        "GIT_ASKPASS",
        "SSH_AUTH_SOCK",
        "PATH",
        "HOME",
        "ANTHROPIC_API_KEY",
        "CLAUDE_CODE_OAUTH_TOKEN",
    ];
    let taken = crate::agent::credential_env(names);
    for gone in &names[..10] {
        assert!(taken.iter().any(|t| t == gone), "{gone} should be removed");
    }
    for kept in &names[10..] {
        assert!(!taken.iter().any(|t| t == kept), "{kept} must stay");
    }
}

// -- Anthropic: cache blocks and cache tokens ------------------------------------------

#[test]
fn anthropic_gets_two_cacheable_system_blocks_and_the_cache_tokens_come_back() {
    use gitpoulpi_hosting::RecordedTransport;
    let reader = Fake::new();
    let ctx = built(&reader, &selection(&["src/a.rs"]), limits());
    let block = ctx.block();
    let transport = Arc::new(RecordedTransport::new().on(
        "/messages",
        200,
        r#"{"content":[{"type":"text","text":"Draft."}],
            "usage":{"cache_read_input_tokens":12000,"cache_creation_input_tokens":300}}"#,
    ));
    let client = crate::AnthropicClient::new(
        transport.clone(),
        "https://api.anthropic.com/v1",
        "m",
        "k".into(),
    );
    let done = block_on(crate::AiClient::complete_request(
        &client,
        &crate::Request {
            system: "BASE + layers",
            context: Some(&block),
            history: &[],
            user: "the conversation",
        },
    ))
    .unwrap();
    assert_eq!(done.text, "Draft.");
    assert_eq!(done.usage.cache_read_tokens, 12_000);
    assert_eq!(done.usage.cache_creation_tokens, 300);
    let body: serde_json::Value =
        serde_json::from_str(transport.requests()[0].body.as_ref().unwrap()).unwrap();
    let system = body["system"].as_array().expect("system is blocks");
    assert_eq!(system.len(), 2);
    assert_eq!(system[0]["text"], "BASE + layers");
    assert_eq!(system[1]["text"], block.text);
    for b in system {
        assert_eq!(b["cache_control"]["type"], "ephemeral");
    }
    assert_eq!(
        body["messages"][0]["content"], "the conversation",
        "what changes every time stays out of the cached blocks"
    );
}

#[test]
fn without_context_anthropic_keeps_a_plain_system_string() {
    use gitpoulpi_hosting::RecordedTransport;
    let transport = Arc::new(RecordedTransport::new().on(
        "/messages",
        200,
        r#"{"content":[{"type":"text","text":"ok"}]}"#,
    ));
    let client = crate::AnthropicClient::new(transport.clone(), "http://x/v1", "m", "k".into());
    let done = block_on(crate::AiClient::complete_request(
        &client,
        &crate::Request {
            system: "S",
            context: None,
            history: &[],
            user: "U",
        },
    ))
    .unwrap();
    assert_eq!(done.usage, crate::Usage::default());
    let body: serde_json::Value =
        serde_json::from_str(transport.requests()[0].body.as_ref().unwrap()).unwrap();
    assert_eq!(body["system"], "S");
}

#[test]
fn an_openai_compatible_server_gets_the_context_after_the_instructions() {
    use gitpoulpi_hosting::RecordedTransport;
    let reader = Fake::new();
    let block = built(&reader, &selection(&["src/a.rs"]), limits()).block();
    let transport = Arc::new(RecordedTransport::new().on(
        "/chat/completions",
        200,
        r#"{"choices":[{"message":{"content":"ok"}}]}"#,
    ));
    let client = crate::OpenAiClient::new(transport.clone(), "http://x/v1", "m", None);
    block_on(crate::AiClient::complete_request(
        &client,
        &crate::Request {
            system: "BASE",
            context: Some(&block),
            history: &[],
            user: "U",
        },
    ))
    .unwrap();
    let body: serde_json::Value =
        serde_json::from_str(transport.requests()[0].body.as_ref().unwrap()).unwrap();
    let system = body["messages"][0]["content"].as_str().unwrap();
    assert!(
        system.starts_with("BASE\n\n"),
        "the stable prefix comes first"
    );
    assert!(system.contains("fn a() {}"));
}

// -- the report ------------------------------------------------------------------------

struct Echo;

#[async_trait::async_trait]
impl crate::AiClient for Echo {
    async fn complete(&self, _system: &str, _user: &str) -> Result<String> {
        Ok("Draft text".into())
    }
}

#[test]
fn the_report_lists_the_parts_and_the_provider_never_publishes() {
    let reader = Fake::new();
    let mut cache = ContextCache::new(30);
    let mut sessions = SessionStore::new();
    let draft = block_on(super::repo_access::draft_reading_with(
        http_job(),
        &Echo,
        plain_request("see src/a.rs"),
        &mut setup(&reader, &mut cache, &mut sessions),
    ))
    .unwrap();
    assert!(draft.context.repository);
    let repo = draft.context.repo.as_ref().unwrap();
    assert_eq!(repo.route, "context");
    assert!(repo.parts.iter().any(|p| p.label == "src/a.rs"));
    assert_eq!(repo.head.len(), 7);
    assert!(repo.bytes > 0 && repo.bytes <= repo.budget);
    assert_eq!(draft.text, "Draft text");
}
