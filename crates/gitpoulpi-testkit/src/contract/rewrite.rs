//! History rewrite, slice S3 (the history-rewrite design):
//! the metadata engine on the current branch. Expected objects are built
//! with plain `git` (`commit-tree`, `hash-object`) so two engines that are
//! wrong the same way cannot agree by accident.

use std::io::Write;
use std::ops::ControlFlow;
use std::process::{Command, Stdio};

use gitpoulpi_core::domain::rewrite::{
    CommitterPolicy, IdentityMatch, IdentityRule, IdentitySet, Recipe, RewriteField,
    RewriteOutcome, RewritePlan, RewriteRefusal, Rule, RuleKind, Scope, SignaturePolicy, WhichDate,
    Who,
};
use gitpoulpi_core::{Error, GitBackend};

use crate::{FixtureRepo, FixtureRepoBuilder};

pub(super) const BASE: i64 = 1_700_000_000;

pub(super) const FAKE_SIGNATURE: &str = "gpgsig -----BEGIN PGP SIGNATURE-----\n \n iQEzBAABCAAdFiEEfake\n =abcd\n -----END PGP SIGNATURE-----\n";

/// `git` in the fixture, with bytes in and bytes out and the user's config
/// kept out (the fixture's own repo-local identity stays).
pub(super) fn git_bytes(
    repo: &FixtureRepo,
    args: &[&str],
    env: &[(&str, &str)],
    stdin: &[u8],
) -> Vec<u8> {
    let mut cmd = Command::new("git");
    cmd.args(args).current_dir(repo.root());
    cmd.env("GIT_CONFIG_NOSYSTEM", "1");
    cmd.env(
        "GIT_CONFIG_GLOBAL",
        if cfg!(windows) { "NUL" } else { "/dev/null" },
    );
    for (k, v) in env {
        cmd.env(k, v);
    }
    let mut child = cmd
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("spawn git");
    let mut pipe = child.stdin.take().unwrap();
    let input = stdin.to_vec();
    let writer = std::thread::spawn(move || {
        let _ = pipe.write_all(&input);
    });
    let out = child.wait_with_output().unwrap();
    writer.join().unwrap();
    assert!(
        out.status.success(),
        "git {} failed: {}",
        args.join(" "),
        String::from_utf8_lossy(&out.stderr)
    );
    out.stdout
}

pub(super) fn git_text(repo: &FixtureRepo, args: &[&str]) -> String {
    String::from_utf8(git_bytes(repo, args, &[], b""))
        .unwrap()
        .trim()
        .to_string()
}

/// A commit object written byte for byte. `extra` is whole header lines
/// (continuations included) placed after the committer line.
pub(super) struct Raw<'a> {
    pub(super) parents: &'a [&'a str],
    pub(super) author: &'a str,
    pub(super) committer: &'a str,
    pub(super) extra: &'a str,
    pub(super) message: &'a [u8],
}

pub(super) fn raw_commit(repo: &FixtureRepo, raw: &Raw<'_>) -> String {
    let tree = git_text(repo, &["rev-parse", "HEAD^{tree}"]);
    let mut object = format!("tree {tree}\n").into_bytes();
    for p in raw.parents {
        object.extend_from_slice(format!("parent {p}\n").as_bytes());
    }
    object.extend_from_slice(
        format!("author {}\ncommitter {}\n", raw.author, raw.committer).as_bytes(),
    );
    object.extend_from_slice(raw.extra.as_bytes());
    object.push(b'\n');
    object.extend_from_slice(raw.message);
    let id = git_bytes(
        repo,
        &[
            "hash-object",
            "-t",
            "commit",
            "-w",
            "--stdin",
            "--literally",
        ],
        &[],
        &object,
    );
    String::from_utf8(id).unwrap().trim().to_string()
}

pub(super) fn alice(i: i64) -> String {
    format!(
        "Alice Example <alice@example.com> {} +0200",
        BASE + i * 3600
    )
}

pub(super) fn bob(i: i64) -> String {
    format!("Bob Example <bob@example.com> {} -0530", BASE + i * 3600)
}

pub(super) fn committer(i: i64) -> String {
    format!(
        "Casey Example <casey@example.com> {} +0000",
        BASE + i * 3600 + 60
    )
}

pub(super) fn point_main_at(repo: &FixtureRepo, id: &str) {
    repo.git(&["update-ref", "refs/heads/main", id]);
}

/// Initial commit, then one raw commit per `(author, extra headers)`.
/// Returns the ids, oldest first (after the initial commit).
pub(super) fn chain(authors: &[(String, &str)]) -> (FixtureRepo, Vec<String>) {
    let repo = FixtureRepoBuilder::new()
        .unwrap()
        .with_initial()
        .unwrap()
        .build();
    let ids = chain_in(&repo, authors);
    (repo, ids)
}

/// The same chain on top of `repo`'s current commit.
pub(super) fn chain_in(repo: &FixtureRepo, authors: &[(String, &str)]) -> Vec<String> {
    let mut parent = repo.git(&["rev-parse", "HEAD"]);
    let mut ids = Vec::new();
    for (i, (author, extra)) in authors.iter().enumerate() {
        let message = format!("commit {i}\n");
        let id = raw_commit(
            repo,
            &Raw {
                parents: &[&parent],
                author,
                committer: &committer(i as i64),
                extra,
                message: message.as_bytes(),
            },
        );
        parent = id.clone();
        ids.push(id);
    }
    point_main_at(repo, &parent);
    ids
}

pub(super) fn identity_rule(email: &str, to: (&str, &str), who: Who) -> Rule {
    Rule::new(RuleKind::Identity(IdentityRule {
        when: IdentityMatch::email(email),
        set: IdentitySet {
            name: Some(to.0.to_string()),
            email: Some(to.1.to_string()),
        },
        apply_to: who,
        include_taggers: false,
        map_trailers: false,
    }))
}

pub(super) fn carol_for_alice() -> Recipe {
    let mut recipe = Recipe::new(
        "alice to carol",
        Scope::default(),
        vec![identity_rule(
            "alice@example.com",
            ("Carol Example", "carol@example.com"),
            Who::Author,
        )],
    );
    recipe.options.signatures = Some(SignaturePolicy::Strip);
    recipe
}

pub(super) fn preview(
    backend: &dyn GitBackend,
    repo: &FixtureRepo,
    recipe: &Recipe,
) -> RewritePlan {
    backend
        .rewrite_preview(&repo.repo_ref(), recipe, &mut |_| ControlFlow::Continue(()))
        .expect("preview")
}

pub(super) fn run(
    backend: &dyn GitBackend,
    repo: &FixtureRepo,
    recipe: &Recipe,
) -> (RewritePlan, RewriteOutcome) {
    let plan = preview(backend, repo, recipe);
    let outcome = backend
        .rewrite_apply(&repo.repo_ref(), &plan)
        .expect("apply");
    (plan, outcome)
}

pub(super) fn cat(repo: &FixtureRepo, id: &str) -> Vec<u8> {
    git_bytes(
        repo,
        &["cat-file", "commit", id],
        &[("GIT_NO_REPLACE_OBJECTS", "1")],
        b"",
    )
}

pub(super) fn split_object(object: &[u8]) -> (String, Vec<u8>) {
    let at = object
        .windows(2)
        .position(|w| w == b"\n\n")
        .expect("a commit has a blank line");
    (
        String::from_utf8_lossy(&object[..at + 1]).into_owned(),
        object[at + 2..].to_vec(),
    )
}

fn header_line(headers: &str, name: &str) -> String {
    headers
        .lines()
        .find(|l| l.starts_with(&format!("{name} ")))
        .unwrap_or_default()
        .to_string()
}

pub(super) fn history(repo: &FixtureRepo) -> Vec<String> {
    git_text(repo, &["rev-list", "--reverse", "--topo-order", "HEAD"])
        .lines()
        .map(str::to_string)
        .collect()
}

pub(super) fn run_dirs(repo: &FixtureRepo) -> Vec<String> {
    let dir = repo.root().join(".git").join("gitpoulpi").join("rewrite");
    match std::fs::read_dir(dir) {
        Ok(entries) => entries
            .map(|e| e.unwrap().file_name().to_string_lossy().into_owned())
            .collect(),
        Err(_) => Vec::new(),
    }
}

/// Every object id in the repository, reachable or not, loose or packed,
/// sorted. Ids rather than the files under `.git/objects`: the files include
/// writers' temporary names (`tmp_obj_*`, `tmp_pack_*`) and move when git
/// repacks in the background, neither of which is an object the code under
/// test created; walking the directory tree also races with both.
pub(super) fn object_files(repo: &FixtureRepo) -> Vec<String> {
    let mut ids: Vec<String> = git_text(
        repo,
        &[
            "cat-file",
            "--batch-all-objects",
            "--batch-check=%(objectname)",
        ],
    )
    .lines()
    .map(str::to_string)
    .collect();
    ids.sort();
    ids
}

pub(super) fn refused(result: Result<impl std::fmt::Debug, Error>, which: RewriteRefusal) {
    match result {
        Err(Error::RewriteRefused(got, _)) => assert_eq!(got, which),
        other => panic!("expected {which:?}, got {other:?}"),
    }
}

/// Alice's commits on a linear branch get Carol as author; every new commit
/// is byte for byte what `commit-tree` makes from the same tree, parent,
/// identities, dates and message.
pub fn check_rewrite_identity_on_a_linear_branch_matches_a_commit_tree_oracle(
    backend: &dyn GitBackend,
) {
    let authors: Vec<(String, &str)> = (0..6)
        .map(|i| (if i % 2 == 0 { alice(i) } else { bob(i) }, ""))
        .collect();
    let (repo, _) = chain(&authors);
    let before = history(&repo);

    let (plan, outcome) = run(backend, &repo, &carol_for_alice());

    let after = history(&repo);
    assert_eq!(after.len(), before.len());
    // The oracle: replay every commit with commit-tree.
    let mut expected_parent: Option<String> = None;
    for (old, new) in before.iter().zip(&after) {
        let (headers, message) = split_object(&cat(&repo, old));
        let tree = git_text(&repo, &["rev-parse", &format!("{old}^{{tree}}")]);
        let author = header_line(&headers, "author");
        let author = author.strip_prefix("author ").unwrap();
        let (ident, date) = author.rsplit_once("> ").unwrap();
        let is_alice = ident.contains("alice@example.com");
        let (name, email) = if is_alice {
            ("Carol Example", "carol@example.com")
        } else {
            let (n, e) = ident.split_once(" <").unwrap();
            (n, e)
        };
        let committer = header_line(&headers, "committer");
        let committer = committer.strip_prefix("committer ").unwrap();
        let (c_ident, c_date) = committer.rsplit_once("> ").unwrap();
        let (c_name, c_email) = c_ident.split_once(" <").unwrap();
        let mut args = vec!["commit-tree", tree.as_str()];
        if let Some(p) = &expected_parent {
            args.push("-p");
            args.push(p);
        }
        let env = [
            ("GIT_AUTHOR_NAME", name),
            ("GIT_AUTHOR_EMAIL", email),
            ("GIT_AUTHOR_DATE", date),
            ("GIT_COMMITTER_NAME", c_name),
            ("GIT_COMMITTER_EMAIL", c_email),
            ("GIT_COMMITTER_DATE", c_date),
        ];
        let made = String::from_utf8(git_bytes(&repo, &args, &env, &message))
            .unwrap()
            .trim()
            .to_string();
        assert_eq!(&made, new, "commit {old} is not what commit-tree builds");
        expected_parent = Some(made);
    }
    assert_eq!(plan.refs.len(), 1);
    assert_eq!(plan.refs[0].name, "refs/heads/main");
    assert_eq!(plan.refs[0].new.unwrap().to_hex(), *after.last().unwrap());
    assert_eq!(outcome.moved.len(), 1);
    // Alice wrote 3 of the 6, and the initial commit is not hers: the first
    // change is the oldest of hers, so everything above it is rewritten.
    assert_eq!(plan.counts.commits_changed, 6);
    assert!(plan.rows.iter().any(|r| r
        .fields
        .iter()
        .any(|f| f.field == RewriteField::AuthorEmail && f.new == "carol@example.com")));
    // Nothing is left of the run but its maps.
    git_text(&repo, &["fsck", "--connectivity-only", "--no-dangling"]);
}

/// Commits below the first change keep their ids (and their signatures);
/// the changed commit and everything above it lose theirs.
pub fn check_rewrite_keeps_the_unchanged_prefix_ids_and_signatures(backend: &dyn GitBackend) {
    let authors = vec![
        (bob(0), FAKE_SIGNATURE),
        (bob(1), ""),
        (alice(2), FAKE_SIGNATURE),
        (bob(3), FAKE_SIGNATURE),
    ];
    let (repo, ids) = chain(&authors);
    let tip_before = git_text(&repo, &["rev-parse", "main"]);

    // No choice made: signed commits would change, so nothing runs.
    let mut recipe = carol_for_alice();
    recipe.options.signatures = None;
    let result =
        backend.rewrite_preview(
            &repo.repo_ref(),
            &recipe,
            &mut |_| ControlFlow::Continue(()),
        );
    refused(result, RewriteRefusal::SignatureChoiceNeeded);
    assert_eq!(git_text(&repo, &["rev-parse", "main"]), tip_before);
    assert!(
        run_dirs(&repo).is_empty(),
        "a refused preview left a run behind"
    );

    // Strip chosen.
    let (plan, _) = run(backend, &repo, &carol_for_alice());
    let after = history(&repo);
    let old = history_of(&repo, &tip_before);
    // initial, c0, c1 are the prefix.
    assert_eq!(&after[..3], &old[..3]);
    assert_eq!(after[1], ids[0]);
    let kept = cat(&repo, &ids[0]);
    assert!(String::from_utf8_lossy(&kept).contains("gpgsig -----BEGIN PGP SIGNATURE-----"));
    for new in &after[3..] {
        let object = String::from_utf8_lossy(&cat(&repo, new)).into_owned();
        assert!(!object.contains("gpgsig"), "{new} kept a stale signature");
    }
    assert_eq!(plan.counts.commits_changed, 2);
    assert_eq!(plan.counts.signed_changed, 2);

    // Signed commits that stay as they are need no choice.
    let (repo, _) = chain(&[(bob(0), FAKE_SIGNATURE), (alice(1), "")]);
    let mut recipe = carol_for_alice();
    recipe.options.signatures = None;
    let (plan, _) = run(backend, &repo, &recipe);
    assert_eq!(plan.counts.commits_changed, 1);
    assert_eq!(plan.counts.signed_changed, 0);
}

pub(super) fn history_of(repo: &FixtureRepo, tip: &str) -> Vec<String> {
    git_text(repo, &["rev-list", "--reverse", "--topo-order", tip])
        .lines()
        .map(str::to_string)
        .collect()
}

/// A merge keeps its second parent (rewritten when it is in the run) and a
/// root commit stays a root commit.
pub fn check_rewrite_keeps_merge_parents_and_the_root_commit(backend: &dyn GitBackend) {
    let repo = FixtureRepoBuilder::new()
        .unwrap()
        .with_initial()
        .unwrap()
        .build();
    let base = repo.git(&["rev-parse", "HEAD"]);
    // main: base <- m1 (bob) ; side: base <- s1 (alice) ; merge(m1, s1) (bob)
    let m1 = raw_commit(
        &repo,
        &Raw {
            parents: &[&base],
            author: &bob(1),
            committer: &committer(1),
            extra: "",
            message: b"main one\n",
        },
    );
    let s1 = raw_commit(
        &repo,
        &Raw {
            parents: &[&base],
            author: &alice(2),
            committer: &committer(2),
            extra: "",
            message: b"side one\n",
        },
    );
    let merge = raw_commit(
        &repo,
        &Raw {
            parents: &[&m1, &s1],
            author: &bob(3),
            committer: &committer(3),
            extra: "",
            message: b"merge side\n",
        },
    );
    point_main_at(&repo, &merge);
    let shape = |repo: &FixtureRepo| {
        git_text(repo, &["log", "--topo-order", "--format=%s|%P|%an", "main"])
            .lines()
            .map(|l| {
                let mut f = l.split('|');
                let subject = f.next().unwrap().to_string();
                let parents = f
                    .next()
                    .unwrap()
                    .split(' ')
                    .filter(|p| !p.is_empty())
                    .count();
                (subject, parents, f.next().unwrap().to_string())
            })
            .collect::<Vec<_>>()
    };
    let before = shape(&repo);

    run(backend, &repo, &carol_for_alice());

    let after = shape(&repo);
    assert_eq!(after.len(), before.len());
    for (b, a) in before.iter().zip(&after) {
        assert_eq!((&a.0, a.1), (&b.0, b.1), "history shape changed");
    }
    let tip = git_text(&repo, &["rev-parse", "main"]);
    let parents = git_text(&repo, &["rev-list", "--parents", "-n", "1", &tip]);
    let parents: Vec<&str> = parents.split(' ').skip(1).collect();
    assert_eq!(parents.len(), 2);
    assert_eq!(parents[0], m1, "the untouched first parent keeps its id");
    assert_ne!(parents[1], s1);
    assert_eq!(
        git_text(&repo, &["log", "-1", "--format=%an <%ae>", parents[1]]),
        "Carol Example <carol@example.com>"
    );
    // The root commit: a rewrite that changes it leaves it parentless.
    let mut recipe = Recipe::new(
        "root",
        Scope::default(),
        vec![identity_rule(
            "testkit@gitpoulpi.local",
            ("Root Example", "root@example.com"),
            Who::Both,
        )],
    );
    recipe.options.signatures = Some(SignaturePolicy::Strip);
    run(backend, &repo, &recipe);
    let roots = git_text(&repo, &["rev-list", "--max-parents=0", "main"]);
    assert_eq!(roots.lines().count(), 1);
    assert_eq!(
        git_text(&repo, &["log", "-1", "--format=%an|%cn", &roots]),
        "Root Example|Root Example"
    );
    git_text(&repo, &["fsck", "--connectivity-only", "--no-dangling"]);
}

/// `encoding`, `mergetag` and unknown headers survive, and a message that is
/// not UTF-8 keeps its bytes exactly (text rules skip it).
pub fn check_rewrite_keeps_encoding_mergetag_and_non_utf8_messages(backend: &dyn GitBackend) {
    let repo = FixtureRepoBuilder::new()
        .unwrap()
        .with_initial()
        .unwrap()
        .build();
    let parent = repo.git(&["rev-parse", "HEAD"]);
    let extra = format!(
        "encoding ISO-8859-1\nmergetag object {parent}\n type commit\n tag v1\n tagger T <t@example.com> 1 +0000\n \n note\nx-custom keep me\n"
    );
    let id = raw_commit(
        &repo,
        &Raw {
            parents: &[&parent],
            author: &alice(1),
            committer: &committer(1),
            extra: &extra,
            message: b"caf\xe9 au lait\n",
        },
    );
    point_main_at(&repo, &id);
    let mut recipe = carol_for_alice();
    recipe.rules.push(Rule::new(RuleKind::MessageReplace(
        gitpoulpi_core::domain::rewrite::TextReplace {
            find: gitpoulpi_core::domain::rewrite::TextPattern::literal("au lait"),
            with: "noir".into(),
        },
    )));
    run(backend, &repo, &recipe);

    let new = git_text(&repo, &["rev-parse", "main"]);
    let (headers, message) = split_object(&cat(&repo, &new));
    assert_eq!(
        message, b"caf\xe9 au lait\n",
        "the latin-1 message was touched"
    );
    assert!(headers.contains("encoding ISO-8859-1\n"));
    assert!(headers.contains(&format!(
        "mergetag object {parent}\n type commit\n tag v1\n tagger T <t@example.com> 1 +0000\n \n note\n"
    )));
    assert!(headers.contains("x-custom keep me\n"));
    assert!(headers.contains("author Carol Example <carol@example.com> "));
    // Same headers in the same order, author line aside.
    let old = split_object(&cat(&repo, &id)).0;
    let names = |h: &str| -> Vec<String> {
        h.lines()
            .filter(|l| !l.starts_with(' '))
            .map(|l| l.split(' ').next().unwrap().to_string())
            .collect()
    };
    assert_eq!(names(&headers), names(&old));
}

/// Utf-8 messages are edited by the message rules.
pub fn check_rewrite_message_rules_edit_utf8_messages(backend: &dyn GitBackend) {
    let (repo, _) = chain(&[(bob(0), ""), (bob(1), "")]);
    let mut recipe = Recipe::new(
        "messages",
        Scope::default(),
        vec![
            Rule::new(RuleKind::MessageReplace(
                gitpoulpi_core::domain::rewrite::TextReplace {
                    find: gitpoulpi_core::domain::rewrite::TextPattern::literal("commit 1"),
                    with: "second".into(),
                },
            )),
            Rule::new(RuleKind::DateShift {
                by_seconds: 3600,
                apply_to: WhichDate::AuthorDate,
            }),
        ],
    );
    recipe.options.signatures = Some(SignaturePolicy::Strip);
    let (plan, _) = run(backend, &repo, &recipe);
    assert_eq!(
        git_text(&repo, &["log", "-1", "--format=%s", "main"]),
        "second"
    );
    // The date rule touches every commit above the initial one's change.
    let tip = git_text(&repo, &["rev-parse", "main"]);
    assert_eq!(
        git_text(
            &repo,
            &["log", "-1", "--date=raw", "--format=%ad|%cd", &tip]
        ),
        format!("{} -0530|{} +0000", BASE + 3600 + 3600, BASE + 3600 + 60)
    );
    assert_eq!(plan.per_rule.len(), 2);
}

/// Rules that need every commit (anonymise, time zone, committer from
/// author) run in order over the whole walk.
pub fn check_rewrite_anonymise_time_zone_and_committer_rules(backend: &dyn GitBackend) {
    use gitpoulpi_core::domain::rewrite::{AnonymiseRule, KeepClock};
    let (repo, _) = chain(&[(alice(0), ""), (bob(1), ""), (alice(2), "")]);
    let mut recipe = Recipe::new(
        "several",
        Scope::default(),
        vec![
            Rule::new(RuleKind::CommitterFromAuthor),
            Rule::new(RuleKind::DateTimezone {
                offset_minutes: 60,
                keep: KeepClock::Instant,
                apply_to: WhichDate::Both,
            }),
            Rule::new(RuleKind::Anonymise(AnonymiseRule::default())),
        ],
    );
    recipe.options.signatures = Some(SignaturePolicy::Strip);
    let (plan, _) = run(backend, &repo, &recipe);
    assert_eq!(plan.per_rule.len(), 3);
    // initial: testkit is contributor 1; alice 2, bob 3 (committers equal the
    // authors after the first rule, so casey never appears).
    let log = git_text(
        &repo,
        &[
            "log",
            "--reverse",
            "--format=%an|%ae|%cn|%cd",
            "--date=raw",
            "main",
        ],
    );
    let lines: Vec<&str> = log.lines().collect();
    assert_eq!(lines.len(), 4);
    assert_eq!(
        lines[1],
        format!(
            "Contributor 2|contributor-2@anonymous.invalid|Contributor 2|{} +0100",
            BASE
        )
    );
    assert!(lines[2].starts_with("Contributor 3|contributor-3@anonymous.invalid|Contributor 3|"));
    assert!(lines[3].starts_with("Contributor 2|"));
    // The instant is kept, only the zone moves.
    assert!(lines[2].ends_with(&format!("{} +0100", BASE + 3600)));
}

/// The committer is kept unless the run asks for "me, now".
pub fn check_rewrite_keeps_the_committer_unless_asked_for_me_now(backend: &dyn GitBackend) {
    let (repo, _) = chain(&[(alice(0), ""), (bob(1), "")]);
    let old_tip = git_text(&repo, &["rev-parse", "main"]);
    let old_committers = git_text(
        &repo,
        &["log", "--format=%cn|%ce|%cd", "--date=raw", "main"],
    );

    run(backend, &repo, &carol_for_alice());
    assert_eq!(
        git_text(
            &repo,
            &["log", "--format=%cn|%ce|%cd", "--date=raw", "main"]
        ),
        old_committers,
        "the committer must be kept"
    );

    // Back to the old history, then "me, now".
    point_main_at(&repo, &old_tip);
    let mut recipe = carol_for_alice();
    recipe.options.committer = CommitterPolicy::MeNow;
    run(backend, &repo, &recipe);
    let lines = git_text(&repo, &["log", "-2", "--format=%cn|%ce|%ct", "main"]);
    for line in lines.lines() {
        let mut f = line.split('|');
        assert_eq!(f.next().unwrap(), "GitPoulpi Testkit");
        assert_eq!(f.next().unwrap(), "testkit@gitpoulpi.local");
        let when: i64 = f.next().unwrap().parse().unwrap();
        assert!(
            when > BASE + 100_000,
            "the committer date was not stamped now"
        );
    }
}

/// A branch that moved since the preview fails the whole apply: nothing
/// changes and no object reaches `objects/`. Discard and a finished apply
/// clean the run directory.
pub fn check_rewrite_apply_refuses_a_moved_branch_and_runs_clean_up(backend: &dyn GitBackend) {
    let (repo, _) = chain(&[(alice(0), ""), (bob(1), "")]);
    let plan = preview(backend, &repo, &carol_for_alice());
    let dirs = run_dirs(&repo);
    assert_eq!(dirs.len(), 1);
    assert!(
        repo.root()
            .join(".git/gitpoulpi/rewrite")
            .join(&dirs[0])
            .join("objects.pack")
            .is_file(),
        "the preview's objects live in a temporary pack"
    );

    repo.git(&["commit", "--allow-empty", "-m", "moved on"]);
    let moved = git_text(&repo, &["rev-parse", "main"]);
    let objects = object_files(&repo);
    refused(
        backend.rewrite_apply(&repo.repo_ref(), &plan),
        RewriteRefusal::Stale,
    );
    assert_eq!(git_text(&repo, &["rev-parse", "main"]), moved);
    assert_eq!(
        object_files(&repo),
        objects,
        "an object leaked into objects/"
    );

    // Discard removes the run, twice is fine.
    backend
        .rewrite_discard(&repo.repo_ref(), &plan.run)
        .unwrap();
    backend
        .rewrite_discard(&repo.repo_ref(), &plan.run)
        .unwrap();
    assert!(run_dirs(&repo).is_empty());

    // A second plan applies; afterwards only the maps remain.
    let plan = preview(backend, &repo, &carol_for_alice());
    backend.rewrite_apply(&repo.repo_ref(), &plan).unwrap();
    let dir = repo.root().join(".git/gitpoulpi/rewrite").join(&plan.run.0);
    assert!(!dir.join("objects.pack").exists());
    let map = std::fs::read_to_string(dir.join("commit-map")).unwrap();
    assert!(map.lines().count() >= 2, "a header and one line per commit");
    let moved_ref = std::fs::read_to_string(dir.join("ref-map")).unwrap();
    assert!(moved_ref.contains("refs/heads/main"));
    // A plan that ran cannot run twice.
    refused(
        backend.rewrite_apply(&repo.repo_ref(), &plan),
        RewriteRefusal::Stale,
    );
}

/// Canceling the preview leaves no run behind; invalid recipes and a
/// repository mid-operation are refused before anything is written.
pub fn check_rewrite_preview_cancel_and_refusals_leave_nothing(backend: &dyn GitBackend) {
    let (repo, _) = chain(&[(alice(0), ""), (bob(1), "")]);
    let tip = git_text(&repo, &["rev-parse", "main"]);

    let result = backend.rewrite_preview(&repo.repo_ref(), &carol_for_alice(), &mut |_| {
        ControlFlow::Break(())
    });
    assert!(matches!(result, Err(Error::Canceled { .. })), "{result:?}");
    assert!(run_dirs(&repo).is_empty());

    let mut recipe = carol_for_alice();
    recipe.rules.clear();
    refused(
        backend.rewrite_preview(
            &repo.repo_ref(),
            &recipe,
            &mut |_| ControlFlow::Continue(()),
        ),
        RewriteRefusal::NoRule,
    );

    std::fs::write(repo.root().join(".git/MERGE_HEAD"), format!("{tip}\n")).unwrap();
    refused(
        backend.rewrite_preview(&repo.repo_ref(), &carol_for_alice(), &mut |_| {
            ControlFlow::Continue(())
        }),
        RewriteRefusal::OperationInProgress,
    );
    std::fs::remove_file(repo.root().join(".git/MERGE_HEAD")).unwrap();
    assert!(run_dirs(&repo).is_empty());
    assert_eq!(git_text(&repo, &["rev-parse", "main"]), tip);

    // Nothing to change: a plan that moves no ref, and apply is a no-op.
    let mut recipe = carol_for_alice();
    recipe.rules = vec![identity_rule(
        "nobody@example.com",
        ("N", "n@example.com"),
        Who::Both,
    )];
    let (plan, outcome) = run(backend, &repo, &recipe);
    assert_eq!(plan.counts.commits_changed, 0);
    assert!(plan.refs.is_empty());
    assert!(outcome.moved.is_empty());
    assert_eq!(git_text(&repo, &["rev-parse", "main"]), tip);
}

/// A `refs/replace/*` must never leak a replaced object into the new
/// history: the engine reads the real objects.
pub fn check_rewrite_ignores_replace_refs(backend: &dyn GitBackend) {
    let (repo, ids) = chain(&[(alice(0), ""), (bob(1), "")]);
    // Replace alice's commit by one with another author.
    let fake = raw_commit(
        &repo,
        &Raw {
            parents: &[&repo.git(&["rev-parse", &format!("{}^", ids[0])])],
            author: &bob(0),
            committer: &committer(0),
            extra: "",
            message: b"commit 0\n",
        },
    );
    repo.git(&["replace", &ids[0], &fake]);

    run(backend, &repo, &carol_for_alice());

    let after = git_text(
        &repo,
        &[
            "--no-replace-objects",
            "log",
            "--format=%an|%s",
            "--reverse",
            "main",
        ],
    );
    assert!(
        after.contains("Carol Example|commit 0"),
        "the replace ref hid alice's commit from the rule: {after}"
    );
}

/// Appends `count` commits to `main` with one `git fast-import`, alternating
/// Alice and Bob as authors. For the spawn-budget and performance tests,
/// where a commit per commit would take minutes.
pub fn import_linear_history(repo: &FixtureRepo, count: usize) {
    let mut stream = String::with_capacity(count * 200);
    for i in 0..count {
        let who = if i % 2 == 0 {
            "Alice Example <alice@example.com>"
        } else {
            "Bob Example <bob@example.com>"
        };
        let when = BASE + (i as i64) * 600;
        let message = format!("change {i}\n");
        let content = format!("{i}\n");
        stream.push_str("commit refs/heads/main\n");
        stream.push_str(&format!("author {who} {when} +0000\n"));
        stream.push_str(&format!(
            "committer Casey Example <casey@example.com> {when} +0000\n"
        ));
        stream.push_str(&format!("data {}\n{message}", message.len()));
        if i == 0 {
            stream.push_str("from refs/heads/main^0\n");
        }
        stream.push_str(&format!(
            "M 100644 inline f.txt\ndata {}\n{content}\n",
            content.len()
        ));
    }
    stream.push_str("done\n");
    git_bytes(
        repo,
        &["fast-import", "--quiet", "--done"],
        &[],
        stream.as_bytes(),
    );
    repo.git(&["reset", "--hard", "-q", "main"]);
}
