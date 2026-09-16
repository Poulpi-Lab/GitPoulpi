//! `.mailmap`, slice S16 (the history-rewrite design): the
//! Log shows mapped authors (git's `%aN`/`%aE`, libgit2's `Mailmap`), the
//! backends resolve identities alike, the Identities list counts every pair,
//! "Merge into..." appends lines, and the `Mailmap` rule rewrites history to
//! what `git log --use-mailmap` shows. Expected values come from plain `git`.

use gitpoulpi_core::domain::rewrite::{
    Ident, Recipe, Rule, RuleKind, Scope, ScopeOptions, SignaturePolicy, Who,
};
use gitpoulpi_core::domain::LogOptions;
use gitpoulpi_core::GitBackend;

use super::rewrite::{chain, committer, git_text, run};
use crate::FixtureRepo;

const BASE: i64 = 1_700_000_000;

/// One person under three emails, a second under one, and every `.mailmap`
/// form: email only, email to email, name and email, name-specific.
const MAILMAP: &str = "\
# people
Alex Doe <alex@example.org> <alex@old.example.com>
<alex.work@example.org> <alex@work.example.com>
Alex Doe <alex@example.org> Alexander <alex@laptop.example.com>
Robert Roe <robert@example.org> Bob <bob@example.com>
";

fn author(name: &str, email: &str, i: i64) -> String {
    format!("{name} <{email}> {} +0100", BASE + i * 60)
}

/// Six commits by five identities (one pair is not in the `.mailmap`).
fn repo_with_mailmap() -> FixtureRepo {
    let authors: Vec<(String, &str)> = [
        ("Alex Doe", "alex@example.org"),
        ("alex", "ALEX@OLD.example.com"),
        ("Alex W", "alex@work.example.com"),
        ("Alexander", "alex@laptop.example.com"),
        ("Bob", "bob@example.com"),
        ("Carol Example", "carol@example.com"),
    ]
    .iter()
    .enumerate()
    .map(|(i, (n, e))| (author(n, e, i as i64), ""))
    .collect();
    let (repo, _) = chain(&authors);
    std::fs::write(repo.root().join(".mailmap"), MAILMAP).unwrap();
    repo
}

fn names(graph: &gitpoulpi_core::domain::CommitGraph) -> Vec<(String, String)> {
    graph
        .nodes
        .iter()
        .map(|n| (n.author_name.clone(), n.author_email.clone()))
        .collect()
}

fn oracle(repo: &FixtureRepo, format: &str) -> Vec<String> {
    git_text(
        repo,
        &["log", "--topo-order", &format!("--format={format}")],
    )
    .lines()
    .map(str::to_string)
    .collect()
}

fn split(line: &str) -> (String, String) {
    let (n, e) = line.split_once('\u{1}').unwrap();
    (n.to_string(), e.to_string())
}

/// The Log shows authors through the `.mailmap` when asked, the raw ones
/// otherwise, exactly what `git log --format=%aN%x01%aE` prints.
pub fn check_log_shows_mailmapped_authors_like_git(backend: &dyn GitBackend) {
    let repo = repo_with_mailmap();
    let raw = backend
        .log(&repo.repo_ref(), &LogOptions::default())
        .unwrap();
    assert_eq!(
        names(&raw),
        oracle(&repo, "%an%x01%ae")
            .iter()
            .map(|l| split(l))
            .collect::<Vec<_>>(),
        "without the setting the authors are the stored ones"
    );
    let shown = backend
        .log(
            &repo.repo_ref(),
            &LogOptions {
                use_mailmap: true,
                ..LogOptions::default()
            },
        )
        .unwrap();
    assert_eq!(
        names(&shown),
        oracle(&repo, "%aN%x01%aE")
            .iter()
            .map(|l| split(l))
            .collect::<Vec<_>>(),
        "with the setting the authors are what git shows"
    );
    // Pin a few so an oracle that maps nothing cannot pass.
    let shown = names(&shown);
    assert!(shown.contains(&("Alex Doe".into(), "alex@example.org".into())));
    assert!(shown.contains(&("Robert Roe".into(), "robert@example.org".into())));
    assert!(shown.contains(&("Carol Example".into(), "carol@example.com".into())));
    assert!(
        shown.contains(&("Alex W".into(), "alex.work@example.org".into())),
        "an email-only entry keeps the commit's name: {shown:?}"
    );
}

/// `mailmap_resolve` answers what `git check-mailmap` does, on both backends.
pub fn check_mailmap_resolve_matches_git_check_mailmap(backend: &dyn GitBackend) {
    let repo = repo_with_mailmap();
    let asked = [
        Ident::new("alex", "alex@old.example.com"),
        Ident::new("alex", "ALEX@old.example.com"),
        Ident::new("Alexander", "alex@laptop.example.com"),
        Ident::new("Someone Else", "alex@laptop.example.com"),
        Ident::new("bob", "bob@example.com"),
        Ident::new("Carol Example", "carol@example.com"),
        Ident::new("Alex W", "alex@work.example.com"),
    ];
    let got = backend.mailmap_resolve(&repo.repo_ref(), &asked).unwrap();
    assert_eq!(got.len(), asked.len());
    for (old, new) in asked.iter().zip(&got) {
        let out = git_text(
            &repo,
            &["check-mailmap", &format!("{} <{}>", old.name, old.email)],
        );
        let expected = out.rsplit_once(" <").unwrap();
        assert_eq!(
            (new.name.as_str(), new.email.as_str()),
            (expected.0, expected.1.trim_end_matches('>')),
            "{old:?}"
        );
    }
    assert_eq!(got[0], Ident::new("Alex Doe", "alex@example.org"));
    assert_eq!(got[3], asked[3], "a name that does not match stays");
    assert!(backend
        .mailmap_resolve(&repo.repo_ref(), &[])
        .unwrap()
        .is_empty());
}

/// The Identities list: every pair of the scope with its counts as author
/// and as committer, most commits first, and what the `.mailmap` maps it to.
pub fn check_mailmap_identities_count_pairs_and_show_what_is_mapped(backend: &dyn GitBackend) {
    let repo = repo_with_mailmap();
    let list = backend
        .mailmap_identities(
            &repo.repo_ref(),
            &Scope::default(),
            &ScopeOptions::default(),
        )
        .unwrap();
    assert!(list.has_mailmap);
    // The initial commit plus six.
    assert_eq!(list.commits, 7);
    let row = |name: &str, email: &str| {
        list.identities
            .iter()
            .find(|i| i.name == name && i.email == email)
            .unwrap_or_else(|| panic!("no row for {name} <{email}>: {:?}", list.identities))
    };
    let casey = row("Casey Example", "casey@example.com");
    assert_eq!((casey.as_author, casey.as_committer), (0, 6));
    assert!(casey.mapped_to.is_none());
    let alexander = row("Alexander", "alex@laptop.example.com");
    assert_eq!((alexander.as_author, alexander.as_committer), (1, 0));
    assert_eq!(
        alexander.mapped_to,
        Some(Ident::new("Alex Doe", "alex@example.org"))
    );
    assert!(row("Carol Example", "carol@example.com")
        .mapped_to
        .is_none());
    // Most commits first: the committer of six leads.
    assert_eq!(list.identities[0].name, "Casey Example");
    let total_author: u64 = list.identities.iter().map(|i| i.as_author).sum();
    assert_eq!(total_author, 7);
}

/// "Merge into..." appends one line per identity, once, nothing else changes
/// and the lines map what they say; bad input writes nothing.
pub fn check_mailmap_add_appends_lines_once_and_refuses_bad_input(backend: &dyn GitBackend) {
    let repo = repo_with_mailmap();
    let before = std::fs::read_to_string(repo.root().join(".mailmap")).unwrap();
    let canonical = Ident::new("Carol Canon", "carol@example.org");
    let from = [
        Ident::new("Carol Example", "carol@example.com"),
        Ident::new("carol", "carol@home.example.com"),
        canonical.clone(),
    ];
    let done = backend
        .mailmap_add(&repo.repo_ref(), &canonical, &from, None)
        .unwrap();
    assert_eq!((done.added, done.file.as_str()), (2, ".mailmap"));
    let after = std::fs::read_to_string(repo.root().join(".mailmap")).unwrap();
    assert!(after.starts_with(&before), "the old lines are untouched");
    assert_eq!(after.lines().count(), before.lines().count() + 2);
    let got = backend
        .mailmap_resolve(&repo.repo_ref(), &from[..2])
        .unwrap();
    assert_eq!(got, vec![canonical.clone(), canonical.clone()]);
    // Again: nothing new.
    let again = backend
        .mailmap_add(&repo.repo_ref(), &canonical, &from, None)
        .unwrap();
    assert_eq!(again.added, 0);
    assert_eq!(
        std::fs::read_to_string(repo.root().join(".mailmap")).unwrap(),
        after
    );
    // A new file in a folder, and refusals.
    let sub = backend
        .mailmap_add(
            &repo.repo_ref(),
            &canonical,
            &from[..1],
            Some("sub/.mailmap"),
        )
        .unwrap();
    assert_eq!((sub.added, sub.file.as_str()), (1, "sub/.mailmap"));
    for bad in [
        backend.mailmap_add(
            &repo.repo_ref(),
            &Ident::new("<x>", "x@example.org"),
            &from[..1],
            None,
        ),
        backend.mailmap_add(&repo.repo_ref(), &Ident::new("", ""), &from[..1], None),
        backend.mailmap_add(&repo.repo_ref(), &canonical, &from[..1], Some("notes.txt")),
        backend.mailmap_add(
            &repo.repo_ref(),
            &canonical,
            &from[..1],
            Some("../.mailmap"),
        ),
    ] {
        assert!(bad.is_err());
    }
    assert_eq!(
        std::fs::read_to_string(repo.root().join(".mailmap")).unwrap(),
        after
    );
}

/// The `Mailmap` rule rewrites authors and committers to what `git log
/// --use-mailmap` shows, and nothing else.
pub fn check_rewrite_mailmap_rule_matches_git_use_mailmap(backend: &dyn GitBackend) {
    let repo = repo_with_mailmap();
    // A committer the file maps too.
    let before_authors = oracle(&repo, "%aN%x01%aE");
    let before_committers = oracle(&repo, "%cN%x01%cE");
    assert!(committer(0).contains("casey@example.com"));
    let before_messages = oracle(&repo, "%s");
    let before_trees = oracle(&repo, "%T");
    let before_dates = oracle(&repo, "%ad%x01%cd");
    let mut recipe = Recipe::new(
        "mailmap",
        Scope::default(),
        vec![Rule::new(RuleKind::Mailmap {
            apply_to: Who::Both,
            file: None,
        })],
    );
    recipe.options.signatures = Some(SignaturePolicy::Strip);
    let (plan, _) = run(backend, &repo, &recipe);
    assert_eq!(oracle(&repo, "%an%x01%ae"), before_authors);
    assert_eq!(oracle(&repo, "%cn%x01%ce"), before_committers);
    assert_eq!(oracle(&repo, "%s"), before_messages);
    assert_eq!(oracle(&repo, "%T"), before_trees);
    assert_eq!(oracle(&repo, "%ad%x01%cd"), before_dates);
    assert!(plan.counts.commits_changed >= 4, "{:?}", plan.counts);
    assert_eq!(plan.per_rule[0].commits, 4);
    // Nothing is left to map: a second run changes nothing.
    let again = super::rewrite::preview(backend, &repo, &recipe);
    assert_eq!(again.counts.commits_changed, 0);
}

/// A given file is applied instead of the worktree's, and a missing one is a
/// typed refusal that changes nothing.
pub fn check_rewrite_mailmap_rule_reads_a_given_file_and_refuses_a_missing_one(
    backend: &dyn GitBackend,
) {
    let repo = repo_with_mailmap();
    std::fs::write(
        repo.root().join("other.mailmap"),
        "Carol Canon <carol@example.org> <carol@example.com>\n",
    )
    .unwrap();
    let rule = |file: &str| {
        let mut r = Recipe::new(
            "mailmap",
            Scope::default(),
            vec![Rule::new(RuleKind::Mailmap {
                apply_to: Who::Author,
                file: Some(file.into()),
            })],
        );
        r.options.signatures = Some(SignaturePolicy::Strip);
        r
    };
    let (plan, _) = run(backend, &repo, &rule("other.mailmap"));
    assert_eq!(plan.per_rule[0].commits, 1);
    let authors = oracle(&repo, "%an%x01%ae");
    assert!(authors.contains(&"Carol Canon\u{1}carol@example.org".to_string()));
    // Alexander was not touched: the worktree file was not read.
    assert!(authors.contains(&"Alexander\u{1}alex@laptop.example.com".to_string()));
    let before = oracle(&repo, "%H");
    let result = backend.rewrite_preview(&repo.repo_ref(), &rule("missing.mailmap"), &mut |_| {
        std::ops::ControlFlow::Continue(())
    });
    assert!(result.is_err());
    assert_eq!(oracle(&repo, "%H"), before);
}
