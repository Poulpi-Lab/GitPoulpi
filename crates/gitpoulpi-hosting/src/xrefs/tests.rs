//! Table tests per host grammar: positives and the negatives of plan 4c.

use gitpoulpi_config::ApiFamily;

use super::*;
use crate::refs::{IssueLinker, KnownCard, PullLinker};

fn issue_linker(family: ApiFamily, base: &str, project: &str) -> IssueLinker {
    IssueLinker {
        family,
        web_base: base.into(),
        project: project.into(),
        project_keys: Vec::new(),
        cards: Vec::new(),
    }
}

fn pull_linker(family: ApiFamily, base: &str, project: &str) -> PullLinker {
    PullLinker {
        family,
        web_base: base.into(),
        project: project.into(),
    }
}

/// A host context with its own repository linked as ISSUES and PULLS.
fn host(family: ApiFamily, base: &str, project: &str) -> XrefContext {
    XrefContext {
        family: Some(family),
        web_base: base.into(),
        project: project.into(),
        repo_links: RefContext {
            issues: Some(issue_linker(family, base, project)),
            pulls: Some(pull_linker(family, base, project)),
        },
        local_notes: Vec::new(),
    }
}

/// Same host, nothing linked: tokens resolve to web pages only.
fn unlinked(family: ApiFamily, base: &str, project: &str) -> XrefContext {
    XrefContext {
        repo_links: RefContext::default(),
        ..host(family, base, project)
    }
}

fn repr(t: &XrefTarget) -> String {
    match t {
        XrefTarget::InApp(i) => format!("app:{}", i.key),
        XrefTarget::Web(u) => format!("web:{u}"),
        XrefTarget::Mention { login, url } => format!("@{login}={}", url.as_deref().unwrap_or("-")),
        XrefTarget::Commit { sha, url } => format!("sha:{sha}={}", url.as_deref().unwrap_or("-")),
        XrefTarget::Chip { kind, name } => format!("chip:{kind:?}:{name}"),
    }
}

fn found(text: &str, ctx: &XrefContext) -> Vec<(String, String)> {
    scan(text, ctx)
        .into_iter()
        .map(|x| (x.text, repr(&x.target)))
        .collect()
}

fn check(ctx: &XrefContext, cases: &[(&str, &[(&str, &str)])]) {
    for (input, want) in cases {
        let want: Vec<(String, String)> = want
            .iter()
            .map(|(a, b)| ((*a).to_string(), (*b).to_string()))
            .collect();
        assert_eq!(found(input, ctx), want, "{input:?}");
    }
}

fn none(ctx: &XrefContext, inputs: &[&str]) {
    for input in inputs {
        assert_eq!(found(input, ctx), Vec::new(), "{input:?}");
    }
}

const GH: &str = "https://github.com";

#[test]
fn github_positives() {
    let ctx = host(ApiFamily::Github, GH, "me/thing");
    check(
        &ctx,
        &[
            ("fixes #7", &[("#7", "app:#7")]),
            ("see GH-8 now", &[("GH-8", "app:#8")]),
            (
                "see octo/cat#9",
                &[("octo/cat#9", "web:https://github.com/octo/cat/issues/9")],
            ),
            ("see me/thing#9", &[("me/thing#9", "app:#9")]),
            (
                "ping @alice and @org/core",
                &[
                    ("@alice", "@alice=https://github.com/alice"),
                    (
                        "@org/core",
                        "@org/core=https://github.com/orgs/org/teams/core",
                    ),
                ],
            ),
            (
                "at abc1234 done",
                &[(
                    "abc1234",
                    "sha:abc1234=https://github.com/me/thing/commit/abc1234",
                )],
            ),
            (
                "octo/cat@abc1234",
                &[(
                    "octo/cat@abc1234",
                    "sha:abc1234=https://github.com/octo/cat/commit/abc1234",
                )],
            ),
            (
                "(see https://github.com/me/thing/issues/12.)",
                &[("https://github.com/me/thing/issues/12", "app:#12")],
            ),
            (
                "https://github.com/other/repo/pull/3",
                &[(
                    "https://github.com/other/repo/pull/3",
                    "web:https://github.com/other/repo/pull/3",
                )],
            ),
            (
                "https://github.com/me/thing/pull/5",
                &[("https://github.com/me/thing/pull/5", "app:#5")],
            ),
            (
                "https://github.com/me/thing/commit/abc1234def",
                &[(
                    "https://github.com/me/thing/commit/abc1234def",
                    "sha:abc1234def=https://github.com/me/thing/commit/abc1234def",
                )],
            ),
        ],
    );
}

#[test]
fn github_unlinked_falls_back_to_web() {
    let ctx = unlinked(ApiFamily::Github, GH, "me/thing");
    check(
        &ctx,
        &[("#7", &[("#7", "web:https://github.com/me/thing/issues/7")])],
    );
}

#[test]
fn negatives_everywhere() {
    for family in [
        ApiFamily::Github,
        ApiFamily::Gitlab,
        ApiFamily::Gitea,
        ApiFamily::AzureDevops,
        ApiFamily::Redmine,
        ApiFamily::Jira,
        ApiFamily::Linear,
        ApiFamily::YouTrack,
    ] {
        let ctx = host(family, "https://h.example", "g/p");
        none(
            &ctx,
            &[
                "C#1 and F#2",
                "UTF-8 SHA-256 ISO-8601 x-1",
                "mail me@example.com or a.b@c.org",
                "`#7` and `GH-8`",
                "```\n#7 abc1234\n```",
                "https://x.example/page#12 and https://y.example/a?b=#3",
                "decade facade 1234567 deadbeef",
                "[#7](https://x.example/z)",
                "![#7](a.png)",
                "versionless 50% and %s",
            ],
        );
    }
}

#[test]
fn code_does_not_hide_the_rest() {
    let ctx = host(ApiFamily::Github, GH, "me/thing");
    check(
        &ctx,
        &[
            ("`#7` then #8", &[("#8", "app:#8")]),
            ("```\n#7\n```\n#9", &[("#9", "app:#9")]),
            ("~~~\n#7\n~~~\n#9", &[("#9", "app:#9")]),
            ("``a ` #7``  #9", &[("#9", "app:#9")]),
            ("unclosed ` #7", &[("#7", "app:#7")]),
        ],
    );
}

#[test]
fn offsets_are_char_offsets() {
    let ctx = host(ApiFamily::Github, GH, "me/thing");
    let hits = scan("\u{4e2d}\u{6587} #7 \u{1f600} #8", &ctx);
    let spans: Vec<(usize, usize)> = hits.iter().map(|x| (x.start, x.end)).collect();
    assert_eq!(spans, vec![(3, 5), (8, 10)]);
}

#[test]
fn gitlab() {
    let base = "https://gl.example";
    let ctx = host(ApiFamily::Gitlab, base, "grp/proj");
    check(
        &ctx,
        &[
            ("#3 and !4", &[("#3", "app:#3"), ("!4", "app:!4")]),
            (
                "other/thing#5 and other/thing!6",
                &[
                    (
                        "other/thing#5",
                        "web:https://gl.example/other/thing/-/issues/5",
                    ),
                    (
                        "other/thing!6",
                        "web:https://gl.example/other/thing/-/merge_requests/6",
                    ),
                ],
            ),
            (
                "sub/dir/thing#5",
                &[(
                    "sub/dir/thing#5",
                    "web:https://gl.example/sub/dir/thing/-/issues/5",
                )],
            ),
            (
                "sibling#2",
                &[("sibling#2", "web:https://gl.example/grp/sibling/-/issues/2")],
            ),
            ("grp/proj!9", &[("grp/proj!9", "app:!9")]),
            (
                "@user.name",
                &[("@user.name", "@user.name=https://gl.example/user.name")],
            ),
            (
                "~bug ~\"needs review\" %v1.0 %\"Q4 plan\" %12",
                &[
                    ("~bug", "chip:Label:bug"),
                    ("~\"needs review\"", "chip:Label:needs review"),
                    ("%v1.0", "chip:Milestone:v1.0"),
                    ("%\"Q4 plan\"", "chip:Milestone:Q4 plan"),
                    ("%12", "chip:Milestone:12"),
                ],
            ),
            (
                "https://gl.example/grp/proj/-/issues/8",
                &[("https://gl.example/grp/proj/-/issues/8", "app:#8")],
            ),
            (
                "https://gl.example/grp/proj/-/merge_requests/9",
                &[("https://gl.example/grp/proj/-/merge_requests/9", "app:!9")],
            ),
            (
                "https://gl.example/grp/proj/-/commit/abc1234",
                &[(
                    "https://gl.example/grp/proj/-/commit/abc1234",
                    "sha:abc1234=https://gl.example/grp/proj/-/commit/abc1234",
                )],
            ),
        ],
    );
    none(&ctx, &["~/dir and ~~struck~~ and ~a", "C#1", "x%d y%s"]);
}

#[test]
fn gitea_and_forgejo() {
    let base = "https://codeberg.org";
    let ctx = host(ApiFamily::Gitea, base, "me/repo");
    check(
        &ctx,
        &[
            ("#7", &[("#7", "app:#7")]),
            ("!8", &[("!8", "web:https://codeberg.org/me/repo/pulls/8")]),
            (
                "you/lib#2",
                &[("you/lib#2", "web:https://codeberg.org/you/lib/issues/2")],
            ),
            ("@bob", &[("@bob", "@bob=https://codeberg.org/bob")]),
            (
                "https://codeberg.org/me/repo/pulls/4",
                &[("https://codeberg.org/me/repo/pulls/4", "app:#4")],
            ),
        ],
    );
}

#[test]
fn azure_devops() {
    let base = "https://dev.azure.com/org";
    let ctx = host(ApiFamily::AzureDevops, base, "proj/repo");
    let guid = "0123abcd-4567-89ab-cdef-0123456789ab";
    let text = format!("#12 and !3 and AB#9 and @<{guid}>");
    let hits = found(&text, &ctx);
    let texts: Vec<&str> = hits.iter().map(|(t, _)| t.as_str()).collect();
    assert_eq!(texts, vec!["#12", "!3", "AB#9", &format!("@<{guid}>")]);
    assert_eq!(hits[3].1, "@user=-");
    assert!(hits[0].1.starts_with("app:"), "{hits:?}");
    none(
        &ctx,
        &[
            "user@example.com",
            "@<nope>",
            "@<0123abcd-4567-89ab-cdef-0123456789a>",
        ],
    );

    let ctx = unlinked(ApiFamily::AzureDevops, base, "proj/repo");
    let hits = found("#12", &ctx);
    assert_eq!(
        hits,
        vec![(
            "#12".to_string(),
            "web:https://dev.azure.com/org/proj/_workitems/edit/12".to_string()
        )]
    );
}

#[test]
fn bitbucket() {
    let base = "https://bitbucket.org";
    let ctx = unlinked(ApiFamily::Bitbucket, base, "ws/repo");
    let uuid = "{0123abcd-4567-89ab-cdef-0123456789ab}";
    check(
        &ctx,
        &[
            (
                "@alice",
                &[("@alice", "@alice=https://bitbucket.org/alice/")],
            ),
            (
                "https://bitbucket.org/ws/repo/pull-requests/12",
                &[(
                    "https://bitbucket.org/ws/repo/pull-requests/12",
                    "web:https://bitbucket.org/ws/repo/pull-requests/12",
                )],
            ),
        ],
    );
    let hits = found(&format!("by {uuid} and @{uuid}"), &ctx);
    assert_eq!(hits.len(), 2, "{hits:?}");
    assert_eq!(hits[0].1, format!("@{uuid}=-"));
    assert_eq!(hits[1].1, format!("@{uuid}=-"));
    none(&ctx, &["#7 !8", "{not-a-uuid}"]);
}

#[test]
fn gerrit() {
    let base = "https://gerrit.example";
    let ctx = unlinked(ApiFamily::Gerrit, base, "proj");
    let id = format!("I{}", "ab12".repeat(10));
    check(
        &ctx,
        &[
            (
                &format!("Change-Id: {id}"),
                &[(&id, &format!("web:https://gerrit.example/q/{id}"))],
            ),
            (
                "https://gerrit.example/c/proj/+/1234",
                &[(
                    "https://gerrit.example/c/proj/+/1234",
                    "web:https://gerrit.example/c/proj/+/1234",
                )],
            ),
        ],
    );
    none(&ctx, &["#7", &format!("I{}", "ab12".repeat(9))]);
}

#[test]
fn key_trackers_known_keys_only() {
    for (family, base) in [
        (ApiFamily::Jira, "https://acme.atlassian.net"),
        (ApiFamily::Linear, "https://linear.app/acme"),
        (ApiFamily::YouTrack, "https://acme.youtrack.cloud"),
    ] {
        let mut ctx = host(family, base, "PROJ");
        ctx.repo_links.pulls = None;
        if let Some(i) = ctx.repo_links.issues.as_mut() {
            i.project_keys = vec!["OPS".into()];
        }
        let hits = found(
            "fix PROJ-12 and OPS-3, not UTF-8 SHA-256 ISO-8601 OTHER-5 or XPROJ-1",
            &ctx,
        );
        let texts: Vec<&str> = hits.iter().map(|(t, _)| t.as_str()).collect();
        assert_eq!(texts, vec!["PROJ-12", "OPS-3"], "{family:?}");
        assert!(hits[0].1.starts_with("app:PROJ-12"), "{hits:?}");
        let hits = found("ping @carol", &ctx);
        assert_eq!(hits.len(), 1, "{family:?}");
        none(&ctx, &["`PROJ-12`", "me@x.org"]);
    }
}

#[test]
fn jira_key_inside_a_github_comment() {
    let mut ctx = host(ApiFamily::Github, GH, "me/thing");
    ctx.repo_links.issues = Some(issue_linker(
        ApiFamily::Jira,
        "https://acme.atlassian.net",
        "PROJ",
    ));
    check(
        &ctx,
        &[(
            "PROJ-4 and #5",
            &[("PROJ-4", "app:PROJ-4"), ("#5", "app:#5")],
        )],
    );
    match &scan("PROJ-4", &ctx)[0].target {
        XrefTarget::InApp(t) => {
            assert_eq!(t.family, ApiFamily::Jira);
            assert_eq!(t.kind, RefKind::Issue);
        }
        other => panic!("{other:?}"),
    }
}

#[test]
fn jira_issue_url_opens_in_app() {
    let mut ctx = host(ApiFamily::Github, GH, "me/thing");
    ctx.repo_links.issues = Some(issue_linker(
        ApiFamily::Jira,
        "https://acme.atlassian.net",
        "PROJ",
    ));
    check(
        &ctx,
        &[(
            "https://acme.atlassian.net/browse/PROJ-4",
            &[("https://acme.atlassian.net/browse/PROJ-4", "app:PROJ-4")],
        )],
    );
}

#[test]
fn redmine() {
    let base = "https://rm.example";
    let ctx = host(ApiFamily::Redmine, base, "site");
    check(
        &ctx,
        &[
            ("#5", &[("#5", "app:#5")]),
            (
                "r123",
                &[(
                    "r123",
                    "web:https://rm.example/projects/site/repository/revisions/123",
                )],
            ),
            ("commit:abc1234", &[("commit:abc1234", "sha:abc1234=-")]),
            ("@dev", &[("@dev", "@dev=-")]),
        ],
    );
    none(&ctx, &["r2d2 and r", "me@x.org"]);
}

#[test]
fn trello() {
    let mut ctx = unlinked(ApiFamily::Trello, "https://trello.com", "board");
    ctx.repo_links.issues = Some(IssueLinker {
        cards: vec![KnownCard {
            id_short: 12,
            url: "https://trello.com/c/AbCd1234".into(),
        }],
        ..issue_linker(ApiFamily::Trello, "https://trello.com", "board")
    });
    check(
        &ctx,
        &[
            (
                "https://trello.com/c/AbCd1234/12-slug",
                &[(
                    "https://trello.com/c/AbCd1234/12-slug",
                    "app:https://trello.com/c/AbCd1234",
                )],
            ),
            ("#12", &[("#12", "app:https://trello.com/c/AbCd1234")]),
            ("@dana", &[("@dana", "@dana=https://trello.com/dana")]),
        ],
    );
    none(&ctx, &["#13", "x@y.com"]);
}

#[test]
fn local_notes() {
    let ctx = XrefContext {
        family: Some(ApiFamily::LocalMarkdown),
        local_notes: vec!["Ideas".into(), "projects/Roadmap".into()],
        ..XrefContext::default()
    };
    check(
        &ctx,
        &[
            ("see [[Ideas]]", &[("[[Ideas]]", "app:Ideas")]),
            (
                "see [[ideas|my list]]",
                &[("[[ideas|my list]]", "app:Ideas")],
            ),
            (
                "see [[Roadmap#Q4]]",
                &[("[[Roadmap#Q4]]", "app:projects/Roadmap")],
            ),
            ("see [[Missing]]", &[]),
            (
                "[the plan](projects/Roadmap.md#top) and [web](https://x.example/a.md)",
                &[(
                    "[the plan](projects/Roadmap.md#top)",
                    "app:projects/Roadmap",
                )],
            ),
            ("`[[Ideas]]`", &[]),
        ],
    );
}

#[test]
fn local_notes_numbers_use_the_linked_source() {
    let mut ctx = XrefContext {
        family: Some(ApiFamily::LocalMarkdown),
        project: "notes".into(),
        ..XrefContext::default()
    };
    ctx.repo_links.issues = Some(issue_linker(ApiFamily::LocalMarkdown, "", "notes"));
    let hits = found("see #3", &ctx);
    assert_eq!(hits.len(), 1, "{hits:?}");
    assert!(hits[0].1.starts_with("app:#3"));
}

#[test]
fn no_overlaps_and_sorted() {
    let ctx = host(ApiFamily::Github, GH, "me/thing");
    let hits = scan(
        "#9 https://github.com/me/thing/issues/2 abc1234 @bob #1 octo/cat#3",
        &ctx,
    );
    for w in hits.windows(2) {
        assert!(w[0].end <= w[1].start, "{hits:?}");
    }
    assert_eq!(hits.len(), 6, "{hits:?}");
}

#[test]
fn no_context_still_finds_commits_only_by_family() {
    let ctx = XrefContext::default();
    assert_eq!(found("#7 @bob PROJ-1", &ctx), Vec::new());
    // A commit id needs no host to be a candidate.
    assert_eq!(
        found("see abc1234", &ctx),
        vec![("abc1234".to_string(), "sha:abc1234=-".to_string())]
    );
}
