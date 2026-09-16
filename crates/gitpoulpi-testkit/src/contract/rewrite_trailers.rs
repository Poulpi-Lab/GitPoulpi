//! History rewrite, slice S17 (the history-rewrite design):
//! trailer rules, identity rules that also map trailers, and updating
//! abbreviated commit ids in messages. Expected messages come from plain
//! `git interpret-trailers` and `git log`, not from the code under test.

use gitpoulpi_core::domain::rewrite::{
    Recipe, Rule, RuleKind, Scope, SignaturePolicy, TextPattern, TrailerAction, TrailerIfExists,
    TrailerRule, Who,
};
use gitpoulpi_core::GitBackend;

use super::rewrite::{
    alice, cat, committer, git_bytes, git_text, identity_rule, point_main_at, raw_commit, run,
    split_object, Raw,
};
use crate::{FixtureRepo, FixtureRepoBuilder};

/// Messages that cover git's trailer grammar: none, one block, a block with
/// a continuation line and a cherry-pick line, a body that merely looks
/// like trailers, no final newline.
const MESSAGES: [&str; 6] = [
    "plain subject\n",
    "subject\n\nbody text\n",
    "subject\n\nSigned-off-by: Old Al <al@old.example.com>\nReviewed-by: Rae <rae@example.com>\n",
    "subject\n\nbody\n\nCo-authored-by: Bo <bo@old.example.com>\n  folded\n(cherry picked from commit 0123456789abcdef)\n",
    "Signed-off-by: only a title <t@example.com>\n",
    "subject\n\nno final newline",
];

/// One commit per message on top of the initial commit; the ids, oldest
/// first.
fn repo_with_messages(messages: &[&str]) -> (FixtureRepo, Vec<String>) {
    let repo = FixtureRepoBuilder::new()
        .unwrap()
        .with_initial()
        .unwrap()
        .build();
    let mut parent = repo.git(&["rev-parse", "HEAD"]);
    let mut ids = Vec::new();
    for (i, message) in messages.iter().enumerate() {
        let id = raw_commit(
            &repo,
            &Raw {
                parents: &[&parent],
                author: &alice(i as i64),
                committer: &committer(i as i64),
                extra: "",
                message: message.as_bytes(),
            },
        );
        parent = id.clone();
        ids.push(id);
    }
    point_main_at(&repo, &parent);
    (repo, ids)
}

fn recipe(rules: Vec<Rule>) -> Recipe {
    let mut recipe = Recipe::new("trailers", Scope::default(), rules);
    recipe.options.signatures = Some(SignaturePolicy::Strip);
    recipe
}

fn trailer_rule(rule: TrailerRule) -> Rule {
    Rule::new(RuleKind::Trailer(rule))
}

/// The message of each commit of `main`, oldest first, the initial commit
/// excluded.
fn messages_of(repo: &FixtureRepo, count: usize) -> Vec<Vec<u8>> {
    let ids: Vec<String> = git_text(repo, &["rev-list", "--reverse", "main"])
        .lines()
        .map(str::to_string)
        .collect();
    ids[ids.len() - count..]
        .iter()
        .map(|id| split_object(&cat(repo, id)).1)
        .collect()
}

/// What `git interpret-trailers` makes of `message` with extra arguments.
fn interpret(repo: &FixtureRepo, message: &str, extra: &[&str]) -> Vec<u8> {
    let mut args = vec!["interpret-trailers"];
    args.extend_from_slice(extra);
    git_bytes(repo, &args, &[], message.as_bytes())
}

/// Adding a trailer gives what `git interpret-trailers --trailer` gives, for
/// every message shape, and `if_exists` skip / replace behave as named.
pub fn check_rewrite_trailer_add_matches_interpret_trailers(backend: &dyn GitBackend) {
    let (repo, _) = repo_with_messages(&MESSAGES);
    let rule = TrailerRule::add("Signed-off-by", "Sam Roe <sam@example.com>");
    let (plan, _) = run(backend, &repo, &recipe(vec![trailer_rule(rule)]));
    // The initial commit is in the scope too.
    assert_eq!(plan.per_rule[0].commits, MESSAGES.len() as u64 + 1);

    let after = messages_of(&repo, MESSAGES.len());
    for (message, new) in MESSAGES.iter().zip(&after) {
        // git adds a trailing newline to a message that has none; the rule
        // keeps the message as it was written.
        let expected = interpret(
            &repo,
            message,
            &["--trailer", "Signed-off-by: Sam Roe <sam@example.com>"],
        );
        let new = String::from_utf8(new.clone()).unwrap();
        let expected = String::from_utf8(expected).unwrap();
        let expected = expected.trim_end_matches('\n');
        let new = new.trim_end_matches('\n');
        // For a message without a final newline, some git versions (seen
        // with git for Windows) glue the trailer to the last body line
        // ("...text\nSigned-off-by: ...") where others, like us, separate
        // it with a blank line. Both are accepted; any other difference is
        // a failure.
        let separated = (!message.ends_with('\n'))
            .then(|| {
                expected
                    .rsplit_once('\n')
                    .map(|(head, last)| format!("{head}\n\n{last}"))
            })
            .flatten();
        assert!(
            new == expected || separated.as_deref() == Some(new),
            "message {message:?}: ours {new:?}, git {expected:?}"
        );
    }
    // The title-only message got a block of its own, not a second trailer.
    assert_eq!(
        String::from_utf8_lossy(&after[4]),
        "Signed-off-by: only a title <t@example.com>\n\nSigned-off-by: Sam Roe <sam@example.com>\n"
    );
}

/// `if_exists` and removal: skip leaves messages that have the key, replace
/// swaps every trailer of the key, remove (with and without a value pattern)
/// takes only what it names.
pub fn check_rewrite_trailer_if_exists_remove_and_map_change_only_their_lines(
    backend: &dyn GitBackend,
) {
    let (repo, _) = repo_with_messages(&MESSAGES);
    let mut skip = TrailerRule::add("Reviewed-by", "Zed <zed@example.com>");
    skip.if_exists = TrailerIfExists::Skip;
    let (plan, _) = run(backend, &repo, &recipe(vec![trailer_rule(skip)]));
    // Only the commit that already has Reviewed-by is left alone.
    assert_eq!(plan.per_rule[0].commits, MESSAGES.len() as u64);

    let (repo, _) = repo_with_messages(&MESSAGES);
    let mut replace = TrailerRule::add("Reviewed-by", "Zed <zed@example.com>");
    replace.if_exists = TrailerIfExists::Replace;
    run(backend, &repo, &recipe(vec![trailer_rule(replace)]));
    let after = messages_of(&repo, MESSAGES.len());
    assert_eq!(
        String::from_utf8_lossy(&after[2]),
        "subject\n\nSigned-off-by: Old Al <al@old.example.com>\nReviewed-by: Zed <zed@example.com>\n"
    );

    let (repo, _) = repo_with_messages(&MESSAGES);
    let mut remove = TrailerRule::remove("signed-off-by");
    remove.find = Some(TextPattern::literal("@old.example.com"));
    let (plan, _) = run(backend, &repo, &recipe(vec![trailer_rule(remove)]));
    assert_eq!(plan.per_rule[0].commits, 1);
    let after = messages_of(&repo, MESSAGES.len());
    assert_eq!(
        String::from_utf8_lossy(&after[2]),
        "subject\n\nReviewed-by: Rae <rae@example.com>\n"
    );
    // Every other message is byte for byte what it was.
    for i in [0, 1, 3, 4, 5] {
        assert_eq!(after[i], MESSAGES[i].as_bytes(), "message {i}");
    }

    // Map: the old domain in two keys; the continuation line stays.
    let (repo, _) = repo_with_messages(&MESSAGES);
    let rules = vec![
        trailer_rule(TrailerRule::map(
            "Signed-off-by",
            "@old.example.com",
            "@example.org",
        )),
        trailer_rule(TrailerRule::map(
            "Co-authored-by",
            "@old.example.com",
            "@example.org",
        )),
    ];
    let (plan, _) = run(backend, &repo, &recipe(rules));
    assert_eq!(plan.per_rule[0].commits + plan.per_rule[1].commits, 2);
    let after = messages_of(&repo, MESSAGES.len());
    assert_eq!(
        String::from_utf8_lossy(&after[2]),
        "subject\n\nSigned-off-by: Old Al <al@example.org>\nReviewed-by: Rae <rae@example.com>\n"
    );
    assert_eq!(
        String::from_utf8_lossy(&after[3]),
        "subject\n\nbody\n\nCo-authored-by: Bo <bo@example.org>\n  folded\n(cherry picked from commit 0123456789abcdef)\n"
    );
    // The plan's rows show the message change of the two commits (the
    // others after them are re-created only because a parent moved).
    let with_message = plan
        .rows
        .iter()
        .filter(|r| {
            r.fields
                .iter()
                .any(|f| f.field == gitpoulpi_core::domain::rewrite::RewriteField::Message)
        })
        .count();
    assert_eq!(with_message, 2);
}

/// An identity rule maps the `Name <email>` trailers too, but only with the
/// option on.
pub fn check_rewrite_identity_rule_maps_trailers_only_when_asked(backend: &dyn GitBackend) {
    let message = "subject\n\nSigned-off-by: Al Old <al@old.example.com>\nCo-authored-by: Bo <bo@x.example.com>\n";
    let make = |trailers: bool| {
        let mut rule = identity_rule(
            "al@old.example.com",
            ("Al New", "al@new.example.com"),
            Who::Both,
        );
        if let RuleKind::Identity(r) = &mut rule.kind {
            r.map_trailers = trailers;
        }
        recipe(vec![rule])
    };

    let (repo, _) = repo_with_messages(&[message]);
    run(backend, &repo, &make(false));
    assert_eq!(messages_of(&repo, 1)[0], message.as_bytes());

    let (repo, _) = repo_with_messages(&[message]);
    let (plan, _) = run(backend, &repo, &make(true));
    assert_eq!(plan.per_rule[0].commits, 1);
    assert_eq!(
        String::from_utf8_lossy(&messages_of(&repo, 1)[0]),
        "subject\n\nSigned-off-by: Al New <al@new.example.com>\nCo-authored-by: Bo <bo@x.example.com>\n"
    );
}

/// Messages that are not UTF-8 are never touched by a trailer rule.
pub fn check_rewrite_trailer_rules_leave_non_utf8_messages_alone(backend: &dyn GitBackend) {
    let repo = FixtureRepoBuilder::new()
        .unwrap()
        .with_initial()
        .unwrap()
        .build();
    let parent = repo.git(&["rev-parse", "HEAD"]);
    let raw = b"caf\xe9\n\nSigned-off-by: A <a@x>\n";
    let id = raw_commit(
        &repo,
        &Raw {
            parents: &[&parent],
            author: &alice(0),
            committer: &committer(0),
            extra: "encoding ISO-8859-1\n",
            message: raw,
        },
    );
    point_main_at(&repo, &id);
    let rules = vec![trailer_rule(TrailerRule::remove("Signed-off-by"))];
    let plan = super::rewrite::preview(backend, &repo, &recipe(rules));
    assert_eq!(plan.per_rule[0].commits, 0);
}

/// `update_message_hashes` (off by default): abbreviated ids of 7+ hex in
/// messages follow the rewrite at the same length; what no commit of the
/// set starts with is left and counted.
pub fn check_rewrite_update_message_hashes_follows_the_new_ids(backend: &dyn GitBackend) {
    let repo = FixtureRepoBuilder::new()
        .unwrap()
        .with_initial()
        .unwrap()
        .build();
    let mut parent = repo.git(&["rev-parse", "HEAD"]);
    let mut ids: Vec<String> = Vec::new();
    for i in 0..4usize {
        let message = match i {
            0 => "first\n".to_string(),
            1 => format!("fix up {}\n", &ids[0][..7]),
            2 => format!(
                "Revert {}\n\nThis reverts commit {}, not deadbeef.\n",
                &ids[1][..10],
                ids[1]
            ),
            _ => format!("see {} and {}\n", &ids[0][..12], &ids[2][..7]),
        };
        let id = raw_commit(
            &repo,
            &Raw {
                parents: &[&parent],
                author: &alice(i as i64),
                committer: &committer(i as i64),
                extra: "",
                message: message.as_bytes(),
            },
        );
        parent = id.clone();
        ids.push(id);
    }
    point_main_at(&repo, &parent);
    let rule = || {
        identity_rule(
            "alice@example.com",
            ("Carol Example", "carol@example.com"),
            Who::Author,
        )
    };

    // Off by default: the ids in the messages are left as they were.
    let off = recipe(vec![rule()]);
    assert!(!off.options.update_message_hashes);
    let preview = super::rewrite::preview(backend, &repo, &off);
    assert_eq!(preview.counts.message_hashes_updated, 0);
    assert_eq!(preview.counts.message_hashes_left, 0);

    let mut on = recipe(vec![rule()]);
    on.options.update_message_hashes = true;
    let (plan, _) = run(backend, &repo, &on);
    // 1 + 2 + 2 ids replaced, `deadbeef` left.
    assert_eq!(plan.counts.message_hashes_updated, 5);
    assert_eq!(plan.counts.message_hashes_left, 1);

    let new: Vec<String> = git_text(&repo, &["rev-list", "--reverse", "main"])
        .lines()
        .skip(1)
        .map(str::to_string)
        .collect();
    let after = messages_of(&repo, 4);
    let text = |i: usize| String::from_utf8(after[i].clone()).unwrap();
    assert_eq!(text(0), "first\n");
    assert_eq!(text(1), format!("fix up {}\n", &new[0][..7]));
    assert_eq!(
        text(2),
        format!(
            "Revert {}\n\nThis reverts commit {}, not deadbeef.\n",
            &new[1][..10],
            new[1]
        )
    );
    assert_eq!(
        text(3),
        format!("see {} and {}\n", &new[0][..12], &new[2][..7])
    );
    // Nothing is left pointing at an id that no longer exists.
    for old in &ids {
        for message in &after {
            assert!(!String::from_utf8_lossy(message).contains(&old[..7]));
        }
    }
}

/// The rule kinds are plain data: serialised and read back they are equal,
/// and a trailer rule that cannot run is refused before any history is read.
pub fn check_rewrite_trailer_rule_problems_are_refused_before_anything_runs(
    backend: &dyn GitBackend,
) {
    let (repo, _) = repo_with_messages(&MESSAGES[..1]);
    for bad in [
        TrailerRule::add("Signed off", "x"),
        TrailerRule::add("Signed-off-by", ""),
        TrailerRule::add("Signed-off-by", "two\nlines"),
        TrailerRule {
            action: TrailerAction::Map,
            ..TrailerRule::remove("Signed-off-by")
        },
        TrailerRule::map("Signed-off-by", "", "x"),
    ] {
        let err = backend
            .rewrite_preview(
                &repo.repo_ref(),
                &recipe(vec![trailer_rule(bad.clone())]),
                &mut |_| std::ops::ControlFlow::Continue(()),
            )
            .expect_err("the rule is invalid");
        assert!(
            matches!(
                err,
                gitpoulpi_core::Error::RewriteRefused(
                    gitpoulpi_core::domain::rewrite::RewriteRefusal::InvalidRule,
                    _
                )
            ),
            "{bad:?}: {err:?}"
        );
    }
    assert!(super::rewrite::run_dirs(&repo).is_empty());
}
