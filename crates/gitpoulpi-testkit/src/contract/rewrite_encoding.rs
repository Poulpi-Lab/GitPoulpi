//! History rewrite, slice S24 (the history-rewrite design):
//! the message-encoding rule. Expected texts are written as bytes here and
//! read back with plain `git`, not with the code under test.

use gitpoulpi_core::domain::rewrite::{Recipe, Rule, RuleKind, Scope, SignaturePolicy};
use gitpoulpi_core::GitBackend;

use super::rewrite::{cat, git_bytes, git_text, point_main_at, run, split_object, BASE};
use crate::FixtureRepo;

fn recipe(names: bool) -> Recipe {
    let mut recipe = Recipe::new(
        "encoding",
        Scope::default(),
        vec![Rule::new(RuleKind::MessageEncoding { names })],
    );
    recipe.options.signatures = Some(SignaturePolicy::Strip);
    recipe
}

fn date(i: i64) -> String {
    format!("{} +0000", BASE + i * 3600)
}

/// One commit on top of the initial one, from raw bytes (names included).
fn commit_bytes(
    repo: &FixtureRepo,
    parent: &str,
    person: &[u8],
    extra: &str,
    message: &[u8],
    i: i64,
) -> String {
    let tree = git_text(repo, &["rev-parse", "HEAD^{tree}"]);
    let mut object = format!("tree {tree}\nparent {parent}\nauthor ").into_bytes();
    object.extend_from_slice(person);
    object.extend_from_slice(format!(" <p@example.com> {}\ncommitter ", date(i)).as_bytes());
    object.extend_from_slice(person);
    object.extend_from_slice(format!(" <p@example.com> {}\n{extra}\n", date(i)).as_bytes());
    object.extend_from_slice(message);
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

fn base_repo() -> (FixtureRepo, String) {
    let repo = crate::FixtureRepoBuilder::new()
        .unwrap()
        .with_initial()
        .unwrap()
        .build();
    let parent = repo.git(&["rev-parse", "HEAD"]);
    (repo, parent)
}

/// A commit with an `encoding` header becomes UTF-8 without the header and
/// reads the same; the names (in the same encoding) follow; other headers
/// stay.
pub fn check_rewrite_encoding_declared(backend: &dyn GitBackend) {
    let (repo, parent) = base_repo();
    let latin = commit_bytes(
        &repo,
        &parent,
        b"Ren\xe9 Example",
        "encoding ISO-8859-1\nx-custom keep me\n",
        b"caf\xe9 au lait\n",
        1,
    );
    // "日本" in Shift_JIS.
    let sjis = commit_bytes(
        &repo,
        &latin,
        b"Sato",
        "encoding Shift_JIS\n",
        b"\x93\xfa\x96\x7b\n",
        2,
    );
    point_main_at(&repo, &sjis);

    let (plan, _) = run(backend, &repo, &recipe(false));
    assert_eq!(plan.counts.commits_changed, 2);
    assert_eq!(plan.counts.encoding_left, 0);

    let ids: Vec<String> = git_text(&repo, &["rev-list", "--reverse", "main"])
        .lines()
        .map(str::to_string)
        .collect();
    let (first, second) = (&ids[ids.len() - 2], &ids[ids.len() - 1]);
    let (headers, message) = split_object(&cat(&repo, first));
    assert!(!headers.contains("encoding"), "{headers}");
    assert!(headers.contains("x-custom keep me\n"));
    assert_eq!(message, "caf\u{e9} au lait\n".as_bytes());
    assert!(headers.contains("author Ren\u{e9} Example <p@example.com> "));
    assert!(headers.contains("committer Ren\u{e9} Example <p@example.com> "));
    let (headers, message) = split_object(&cat(&repo, second));
    assert!(!headers.contains("encoding"));
    assert_eq!(message, "\u{65e5}\u{672c}\n".as_bytes());
    // Plain git agrees: no conversion is needed any more.
    assert_eq!(
        git_text(&repo, &["log", "-1", "--format=%s", first]),
        "caf\u{e9} au lait"
    );
    // The preview shows the header going.
    assert!(plan.rows.iter().all(|r| r
        .fields
        .iter()
        .any(|f| f.field == gitpoulpi_core::domain::rewrite::RewriteField::Encoding)));
}

/// UTF-8 that was mis-decoded as Latin-1 / Windows-1252 is repaired; the
/// plain commits around it keep their ids.
pub fn check_rewrite_encoding_mojibake(backend: &dyn GitBackend) {
    let (repo, parent) = base_repo();
    let plain = commit_bytes(
        &repo,
        &parent,
        b"Pat",
        "",
        "caf\u{e9} is fine\n".as_bytes(),
        1,
    );
    let mangled = commit_bytes(
        &repo,
        &plain,
        b"Pat",
        "",
        "R\u{c3}\u{a9}\u{c3}\u{a9}criture\n\nit\u{e2}\u{20ac}\u{2122}s done\n".as_bytes(),
        2,
    );
    point_main_at(&repo, &mangled);

    let (plan, _) = run(backend, &repo, &recipe(false));
    assert_eq!(plan.counts.commits_changed, 1);
    assert_eq!(plan.counts.encoding_left, 0);
    let ids = git_text(&repo, &["rev-list", "--reverse", "main"]);
    let ids: Vec<&str> = ids.lines().collect();
    assert_eq!(ids[ids.len() - 2], plain, "the plain commit keeps its id");
    let (_, message) = split_object(&cat(&repo, ids[ids.len() - 1]));
    assert_eq!(
        message,
        "R\u{e9}\u{e9}criture\n\nit\u{2019}s done\n".as_bytes()
    );
}

/// A message that merely contains "Ã©" next to real letters is not
/// provably mojibake (the round trip is not exact): left alone, byte for
/// byte, and counted. So is a message that is not UTF-8 and declares
/// nothing.
pub fn check_rewrite_encoding_leaves_unprovable(backend: &dyn GitBackend) {
    let (repo, parent) = base_repo();
    let legit = commit_bytes(
        &repo,
        &parent,
        b"Pat",
        "",
        "\u{c3}\u{a9} is how \u{e9} looks when mangled\n".as_bytes(),
        1,
    );
    let bare = commit_bytes(&repo, &legit, b"Pat", "", b"caf\xe9 without a header\n", 2);
    let unknown = commit_bytes(
        &repo,
        &bare,
        b"Pat",
        "encoding no-such-charset\n",
        b"caf\xe9\n",
        3,
    );
    point_main_at(&repo, &unknown);

    let preview = super::rewrite::preview(backend, &repo, &recipe(false));
    assert_eq!(preview.counts.commits_changed, 0);
    assert_eq!(preview.counts.encoding_left, 3);
    assert!(preview.refs.is_empty(), "nothing to move");
    let before = git_text(&repo, &["rev-parse", "main"]);
    let _ = backend.rewrite_apply(&repo.repo_ref(), &preview);
    assert_eq!(git_text(&repo, &["rev-parse", "main"]), before);
    let (_, message) = split_object(&cat(&repo, &legit));
    assert_eq!(
        message,
        "\u{c3}\u{a9} is how \u{e9} looks when mangled\n".as_bytes()
    );
}

/// The names option repairs a mangled author and committer name; without it
/// the names stay.
pub fn check_rewrite_encoding_names_option(backend: &dyn GitBackend) {
    let build = || {
        let (repo, parent) = base_repo();
        let id = commit_bytes(
            &repo,
            &parent,
            "Ren\u{c3}\u{a9} Example".as_bytes(),
            "",
            "fine message\n".as_bytes(),
            1,
        );
        point_main_at(&repo, &id);
        repo
    };
    let repo = build();
    let plan = super::rewrite::preview(backend, &repo, &recipe(false));
    assert_eq!(plan.counts.commits_changed, 0, "names are left by default");

    let repo = build();
    let (plan, _) = run(backend, &repo, &recipe(true));
    assert_eq!(plan.counts.commits_changed, 1);
    let (headers, message) = split_object(&cat(&repo, &git_text(&repo, &["rev-parse", "main"])));
    assert!(headers.contains("author Ren\u{e9} Example <p@example.com> "));
    assert!(headers.contains("committer Ren\u{e9} Example <p@example.com> "));
    assert_eq!(message, b"fine message\n");
}
