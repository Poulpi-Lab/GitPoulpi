//! History rewrite, slice S14 (the history-rewrite design,
//! "Secrets"): scanning the history for secrets and removing the findings by
//! id. Expected values come from plain `git`. Every token is made up at run
//! time from letters that spell nothing real.

use std::ops::ControlFlow;

use gitpoulpi_core::domain::rewrite::{
    ContentFind, ContentReplace, Recipe, RewriteRefusal, Rule, RuleKind, Scope, SecretFinding,
    SecretScan, SecretText, SignaturePolicy,
};
use gitpoulpi_core::{Error, GitBackend};

use super::rewrite::{git_text, refused, run, run_dirs};
use crate::{FixtureRepo, FixtureRepoBuilder};

/// A made-up body of `n` alphanumerics with enough variety to pass for a key.
fn body(seed: &str, n: usize) -> String {
    seed.chars().cycle().take(n).collect()
}

fn github_token(seed: &str) -> String {
    format!("ghp_{}", body(seed, 36))
}

const SEED_A: &str = "Zk9xQ2mL7vB4nR8tY1wC5dF3hJ6pS0aEgUiX";
const SEED_B: &str = "Tq3Vn8Kc1Yz6Mj5Hd9Wb2Lr7Fx4Pg0SaEuNo";

fn start() -> FixtureRepo {
    FixtureRepoBuilder::new()
        .unwrap()
        .with_initial()
        .unwrap()
        .build()
}

fn put(repo: &FixtureRepo, files: &[(&str, &[u8])], message: &str) -> String {
    for (path, bytes) in files {
        let full = repo.root().join(path);
        if let Some(dir) = full.parent() {
            std::fs::create_dir_all(dir).unwrap();
        }
        std::fs::write(full, bytes).unwrap();
    }
    repo.git(&["add", "-A"]);
    repo.git(&["commit", "-q", "-m", message]);
    git_text(repo, &["rev-parse", "HEAD"])
}

fn scan(backend: &dyn GitBackend, repo: &FixtureRepo, scope: &Scope) -> SecretScan {
    backend
        .rewrite_scan_secrets(&repo.repo_ref(), scope, &mut |_| ControlFlow::Continue(()))
        .expect("scan")
}

fn finding<'a>(scan: &'a SecretScan, id: &str) -> &'a SecretFinding {
    scan.findings
        .iter()
        .find(|f| f.id == id)
        .unwrap_or_else(|| panic!("no finding {id}"))
}

fn remove_findings(ids: &[&str]) -> Recipe {
    let rule = RuleKind::ContentReplace(ContentReplace {
        paths: Vec::new(),
        find: ContentFind::Secrets {
            literals: ids.iter().map(|i| SecretText((*i).to_string())).collect(),
            scan: true,
        },
        with: String::new(),
        max_blob_bytes: None,
    });
    let mut recipe = Recipe::new("remove findings", Scope::default(), vec![Rule::new(rule)]);
    recipe.options.signatures = Some(SignaturePolicy::Strip);
    recipe
}

/// The history a repository keeps for these tests:
/// `c1` adds the token, `c2` edits the file (the token stays, in a new blob),
/// `c3` removes the token from the file, `c4` is unrelated. A second token
/// lives in another file from `c2` to the end, and decoys that are not
/// secrets sit next to them.
struct Planted {
    repo: FixtureRepo,
    token_a: String,
    token_b: String,
    c: [String; 4],
}

fn planted() -> Planted {
    let repo = start();
    let (token_a, token_b) = (github_token(SEED_A), github_token(SEED_B));
    let lock = format!(
        "{{\n  \"packages\": {{ \"node_modules/x\": {{ \"integrity\": \"sha512-{}==\" }} }}\n}}\n",
        body(SEED_A, 86)
    );
    let decoys: Vec<(&str, Vec<u8>)> = vec![
        ("package-lock.json", lock.into_bytes()),
        (
            "ids.txt",
            b"token = \"123e4567-e89b-12d3-a456-426614174000\"\n".to_vec(),
        ),
        (
            "logo.svg.txt",
            format!(
                "data:image/png;base64,{}AKIAZ7VRSQ2OTFNKQXYB{}\n",
                body(SEED_B, 40),
                body(SEED_A, 40)
            )
            .into_bytes(),
        ),
        ("binary.bin", vec![0u8, 1, 2, 3, 0]),
    ];
    let mut files: Vec<(&str, &[u8])> = decoys.iter().map(|(p, b)| (*p, b.as_slice())).collect();
    let cfg1 = format!("# config\nGITHUB_TOKEN={token_a}\nmode=dev\n");
    files.push(("config.env", cfg1.as_bytes()));
    let c1 = put(&repo, &files, "add config");
    let cfg2 = format!("# config\nGITHUB_TOKEN={token_a}\nmode=prod\n");
    let other = format!("token: {token_b}\n");
    let c2 = put(
        &repo,
        &[
            ("config.env", cfg2.as_bytes()),
            ("deploy/ci.yml", other.as_bytes()),
        ],
        "edit config, add ci",
    );
    let c3 = put(
        &repo,
        &[("config.env", b"# config\nGITHUB_TOKEN=\nmode=prod\n")],
        "stop committing the token",
    );
    let c4 = put(&repo, &[("README.md", b"hello\n")], "readme");
    Planted {
        repo,
        token_a,
        token_b,
        c: [c1, c2, c3, c4],
    }
}

/// A planted token is found with the right first and last commit, counts and
/// a redacted preview; the decoys are not findings; the answer holds no
/// value; the scope limits what is read; canceling stops the scan.
pub fn check_rewrite_scan_secrets_finds_a_planted_token_with_its_history(backend: &dyn GitBackend) {
    let p = planted();
    p.repo.git(&["branch", "topic", &p.c[0]]);
    p.repo.git(&["checkout", "-q", "topic"]);
    let token_c = github_token("Hs5Ne2Qa8Dm1Ug7Ox3Jy9Bf4Ww6Ck0RlVtIz");
    put(
        &p.repo,
        &[("topic.txt", format!("key {token_c}\n").as_bytes())],
        "topic work",
    );
    p.repo.git(&["checkout", "-q", "main"]);

    let before = (git_text(&p.repo, &["for-each-ref"]), run_dirs(&p.repo));
    let found = scan(backend, &p.repo, &Scope::default());
    assert_eq!(found.findings.len(), 2, "{:?}", found.findings);
    assert_eq!(
        found.commits, 5,
        "main has the initial commit and four more"
    );
    assert!(found.blobs_skipped_binary >= 1);

    // Never a value: not in the answer, however it is printed.
    for token in [&p.token_a, &p.token_b, &token_c] {
        let json = serde_json::to_string(&found).unwrap();
        assert!(!json.contains(token.as_str()), "{json}");
        assert!(!format!("{found:?}").contains(token.as_str()));
        assert!(!json.contains(&token[4..]), "not even without the prefix");
    }
    let a = found
        .findings
        .iter()
        .find(|f| f.path == "config.env")
        .expect("the token in config.env");
    assert_eq!(a.rule, "github-token");
    assert!(a.id.starts_with("f-") && a.id.len() == 14, "{}", a.id);
    assert_eq!(a.first_commit.to_hex(), p.c[0]);
    assert_eq!(a.last_commit.to_hex(), p.c[1]);
    assert_eq!((a.commits, a.blobs, a.more_paths), (2, 2, 0));
    assert_eq!(
        a.preview,
        format!("ghp_****{}", &p.token_a[p.token_a.len() - 4..])
    );

    let b = found
        .findings
        .iter()
        .find(|f| f.path == "deploy/ci.yml")
        .expect("the token in deploy/ci.yml");
    assert_eq!(b.first_commit.to_hex(), p.c[1]);
    assert_eq!(b.last_commit.to_hex(), p.c[3]);
    assert_eq!((b.commits, b.blobs), (3, 1));

    // The decoys: a lock file hash, a UUID, a base64 image holding an AKIA shape.
    assert!(found.findings.iter().all(|f| !matches!(
        f.path.as_str(),
        "package-lock.json" | "ids.txt" | "logo.svg.txt" | "binary.bin"
    )));

    // A scan reads and writes nothing.
    assert_eq!(
        (git_text(&p.repo, &["for-each-ref"]), run_dirs(&p.repo)),
        before
    );

    // The ids do not change between scans, and the scope matters.
    let again = scan(backend, &p.repo, &Scope::default());
    assert_eq!(again.findings, found.findings);
    let all = scan(backend, &p.repo, &Scope::AllLocal);
    assert_eq!(all.findings.len(), 3, "the topic branch's token too");
    let topic_only = scan(
        backend,
        &p.repo,
        &Scope::Refs {
            refs: vec!["topic".into()],
        },
    );
    let ids: Vec<_> = topic_only
        .findings
        .iter()
        .map(|f| f.path.as_str())
        .collect();
    assert!(
        ids.contains(&"topic.txt") && ids.contains(&"config.env"),
        "{ids:?}"
    );
    assert!(!ids.contains(&"deploy/ci.yml"));

    // Canceling stops it with the typed error.
    let result = backend.rewrite_scan_secrets(&p.repo.repo_ref(), &Scope::default(), &mut |_| {
        ControlFlow::Break(())
    });
    assert!(matches!(result, Err(Error::Canceled { .. })), "{result:?}");
    // No commit: a typed refusal.
    let empty = FixtureRepoBuilder::new().unwrap().build();
    refused(
        backend.rewrite_scan_secrets(&empty.repo_ref(), &Scope::default(), &mut |_| {
            ControlFlow::Continue(())
        }),
        RewriteRefusal::EmptyScope,
    );
}

/// "Remove from history" by finding id: the token is gone from every branch
/// and tag, the other secret stays when only one id is named and goes when
/// none is named, the saved plan and the run directory hold no value, an id
/// that is not in the history is refused, and putting the refs back brings
/// the old history back.
pub fn check_rewrite_removes_scanned_secrets_by_finding_id(backend: &dyn GitBackend) {
    let p = planted();
    let found = scan(backend, &p.repo, &Scope::default());
    let a = found
        .findings
        .iter()
        .find(|f| f.path == "config.env")
        .unwrap()
        .id
        .clone();
    let old_tip = git_text(&p.repo, &["rev-parse", "main"]);

    // An id that names nothing.
    refused(
        backend.rewrite_preview(
            &p.repo.repo_ref(),
            &remove_findings(&["f-000000000000"]),
            &mut |_| ControlFlow::Continue(()),
        ),
        RewriteRefusal::InvalidRule,
    );

    let recipe = remove_findings(&[&a]);
    let (plan, _) = run(backend, &p.repo, &recipe);
    // c1 and c2 held the token; c3 and c4 sit on top of them.
    assert_eq!(plan.counts.commits_changed, 4, "{:?}", plan.counts);

    // The token is gone from every reachable commit; the other one stays.
    let log = git_text(&p.repo, &["log", "-p", "--branches", "--tags"]);
    assert!(
        !log.contains(&p.token_a),
        "the removed token is in the history"
    );
    assert!(log.contains(&p.token_b), "the other token stays");
    assert!(log.contains("GITHUB_TOKEN=***REMOVED***"));
    let blobs = git_text(&p.repo, &["rev-list", "--objects", "--branches", "--tags"]);
    for line in blobs.lines() {
        let id = line.split(' ').next().unwrap();
        if git_text(&p.repo, &["cat-file", "-t", id]) == "blob" {
            let text = git_text(&p.repo, &["cat-file", "-p", id]);
            assert!(!text.contains(&p.token_a), "blob {id}");
        }
    }
    p.repo
        .git(&["fsck", "--connectivity-only", "--no-dangling"]);

    // Nothing the preview left behind, and nothing it returned, has a value.
    assert!(!serde_json::to_string(&plan).unwrap().contains(&p.token_a));
    assert!(!format!("{plan:?}").contains(&p.token_a));
    let dir = p
        .repo
        .root()
        .join(".git/gitpoulpi/rewrite")
        .join(&plan.run.0);
    for entry in std::fs::read_dir(&dir).unwrap() {
        let path = entry.unwrap().path();
        if path.file_name().is_some_and(|n| n == "objects.pack") {
            continue;
        }
        let text = String::from_utf8_lossy(&std::fs::read(&path).unwrap()).into_owned();
        for token in [&p.token_a, &p.token_b] {
            assert!(
                !text.contains(token.as_str()),
                "{} holds a token",
                path.display()
            );
        }
        if path.file_name().is_some_and(|n| n == "plan.json") {
            assert!(text.contains(&a), "the finding id is what the plan keeps");
        }
    }
    let saved = serde_json::to_string(&recipe.saved()).unwrap();
    assert!(saved.contains(&a) && !saved.contains(&p.token_a));

    // A scan of the rewritten history no longer finds it.
    let after = scan(backend, &p.repo, &Scope::default());
    assert!(after.findings.iter().all(|f| f.id != a));
    assert_eq!(after.findings.len(), 1);
    assert_eq!(
        finding(
            &after,
            &found
                .findings
                .iter()
                .find(|f| f.path == "deploy/ci.yml")
                .unwrap()
                .id
        )
        .path,
        "deploy/ci.yml"
    );

    // Undo, as the daemon's journal does it: every ref back at its old value.
    for r in &plan.refs {
        if let Some(new) = r.new {
            p.repo
                .git(&["update-ref", &r.name, &r.old.to_hex(), &new.to_hex()]);
        }
    }
    p.repo.git(&["reset", "-q", "--hard", "HEAD"]);
    assert_eq!(git_text(&p.repo, &["rev-parse", "main"]), old_tip);
    let log = git_text(&p.repo, &["log", "-p", "--branches"]);
    assert!(log.contains(&p.token_a) && log.contains(&p.token_b));
    let restored = scan(backend, &p.repo, &Scope::default());
    assert_eq!(restored.findings, found.findings);

    // No id at all: every finding goes.
    run(backend, &p.repo, &remove_findings(&[]));
    let log = git_text(&p.repo, &["log", "-p", "--branches", "--tags"]);
    assert!(!log.contains(&p.token_a) && !log.contains(&p.token_b));
    assert!(scan(backend, &p.repo, &Scope::default())
        .findings
        .is_empty());
    p.repo
        .git(&["fsck", "--connectivity-only", "--no-dangling"]);
}

/// A scanning rule next to a literal: both go. A history with nothing to
/// find is a rewrite that changes nothing.
pub fn check_rewrite_scan_rule_mixes_ids_and_literals_and_tolerates_no_findings(
    backend: &dyn GitBackend,
) {
    let p = planted();
    let found = scan(backend, &p.repo, &Scope::default());
    let b = found
        .findings
        .iter()
        .find(|f| f.path == "deploy/ci.yml")
        .unwrap()
        .id
        .clone();
    let (_, _) = run(backend, &p.repo, &{
        let mut r = remove_findings(&[&b]);
        if let RuleKind::ContentReplace(c) = &mut r.rules[0].kind {
            if let ContentFind::Secrets { literals, .. } = &mut c.find {
                // An id and a plain literal side by side.
                literals.push(SecretText(p.token_a.clone()));
            }
        }
        r
    });
    let log = git_text(&p.repo, &["log", "-p", "--branches"]);
    assert!(!log.contains(&p.token_a) && !log.contains(&p.token_b));

    // Now nothing is left to find: the rule changes nothing.
    let plan = backend
        .rewrite_preview(&p.repo.repo_ref(), &remove_findings(&[]), &mut |_| {
            ControlFlow::Continue(())
        })
        .expect("a scan that finds nothing is not an error");
    assert_eq!(plan.counts.commits_changed, 0);
    assert!(plan.refs.is_empty());
}
