use std::collections::BTreeMap;
use std::sync::Mutex;

use gitpoulpi_config::IntegrationKind;
use gitpoulpi_hosting::RecordedTransport;

use super::*;

fn block_on<T>(f: impl std::future::Future<Output = T>) -> T {
    tokio::runtime::Builder::new_current_thread()
        .build()
        .unwrap()
        .block_on(f)
}

// -- resolution ------------------------------------------------------------

#[test]
fn the_defaults_are_both_files_on_no_extra_file_and_the_profile_decides() {
    let ctx = resolve(&AiReviewContext::default(), None, &RepoAiReview::default());
    assert_eq!(
        ctx,
        EffectiveReviewContext {
            run_in_repository: RunInRepo::Profile,
            include_claude_md: true,
            include_agents_md: true,
            extra_file: String::new(),
        }
    );
}

#[test]
fn each_level_overrides_the_one_below_field_by_field() {
    let global = AiReviewContext {
        run_in_repository: RunInRepo::No,
        include_claude_md: true,
        include_agents_md: true,
        extra_file: "docs/global.md".into(),
    };
    let repo = RepoAiReview {
        run_in_repository: Some(RunInRepo::Yes),
        include_claude_md: Some(false),
        extra_file: Some("docs/repo.md".into()),
        ..RepoAiReview::default()
    };
    // Global alone.
    let g = resolve(&global, None, &RepoAiReview::default());
    assert_eq!(g.run_in_repository, RunInRepo::No);
    assert!(g.include_claude_md);
    assert_eq!(g.extra_file, "docs/global.md");
    // The repository over the global: only the fields it set.
    let r = resolve(&global, Some(&repo), &RepoAiReview::default());
    assert!(!r.include_claude_md);
    assert!(r.include_agents_md, "inherited");
    assert_eq!(r.run_in_repository, RunInRepo::Yes, "the repository's own");
    assert_eq!(r.extra_file, "docs/repo.md");
    // The run over both.
    let run = RepoAiReview {
        run_in_repository: Some(RunInRepo::Profile),
        include_claude_md: Some(true),
        include_agents_md: Some(false),
        extra_file: Some("https://example.org/a.md".into()),
        ..RepoAiReview::default()
    };
    let w = resolve(&global, Some(&repo), &run);
    assert_eq!(w.run_in_repository, RunInRepo::Profile);
    assert!(w.include_claude_md);
    assert!(!w.include_agents_md);
    assert_eq!(w.extra_file, "https://example.org/a.md");
}

#[test]
fn an_empty_extra_file_of_a_run_clears_the_one_of_the_levels_below() {
    let global = AiReviewContext {
        extra_file: "docs/global.md".into(),
        ..AiReviewContext::default()
    };
    let run = RepoAiReview {
        extra_file: Some(String::new()),
        ..RepoAiReview::default()
    };
    assert_eq!(resolve(&global, None, &run).extra_file, "");
}

#[test]
fn the_no_extra_file_switch_matrix() {
    let global = AiReviewContext {
        extra_file: "docs/global.md".into(),
        ..AiReviewContext::default()
    };
    let none = |extra: Option<&str>, no: Option<bool>| RepoAiReview {
        extra_file: extra.map(String::from),
        no_extra_file: no,
        ..RepoAiReview::default()
    };
    let eff =
        |repo: &RepoAiReview, run: &RepoAiReview| resolve(&global, Some(repo), run).extra_file;
    let blank = RepoAiReview::default();
    // The switch turns the global file off for the repository.
    assert_eq!(eff(&none(None, Some(true)), &blank), "");
    // An empty field means "inherit", and so does an unset or false switch.
    assert_eq!(eff(&none(None, None), &blank), "docs/global.md");
    assert_eq!(eff(&none(None, Some(false)), &blank), "docs/global.md");
    // The switch wins over a file kept beside it.
    assert_eq!(eff(&none(Some("docs/repo.md"), Some(true)), &blank), "");
    // A file of the repository replaces the global one.
    assert_eq!(
        eff(&none(Some("docs/repo.md"), None), &blank),
        "docs/repo.md"
    );
    // The run decides last, in both directions.
    assert_eq!(
        eff(&none(None, Some(true)), &none(Some("run.md"), None)),
        "run.md"
    );
    assert_eq!(
        eff(&none(Some("repo.md"), None), &none(None, Some(true))),
        ""
    );
    // No repository values at all: the global file.
    assert_eq!(resolve(&global, None, &blank).extra_file, "docs/global.md");
}

// -- link parsing ----------------------------------------------------------

fn link(url: &str) -> (LinkKind, String, String, String, String) {
    let l = parse_file_link(url).unwrap_or_else(|| panic!("{url} should parse"));
    (l.kind, l.host, l.project, l.rev, l.path)
}

#[test]
fn github_gitlab_and_gitea_blob_links_are_read() {
    assert_eq!(
        link("https://github.com/acme/app/blob/main/docs/REVIEW.md?plain=1#L10"),
        (
            LinkKind::Github,
            "github.com".into(),
            "acme/app".into(),
            "main".into(),
            "docs/REVIEW.md".into()
        )
    );
    assert_eq!(
        link("https://gitlab.corp.example/group/sub/proj/-/blob/v1.2/a%20b/RULES.md"),
        (
            LinkKind::Gitlab,
            "gitlab.corp.example".into(),
            "group/sub/proj".into(),
            "v1.2".into(),
            "a b/RULES.md".into()
        )
    );
    assert_eq!(
        link("https://codeberg.org/me/app/src/branch/dev/CONTRIBUTING.md"),
        (
            LinkKind::Gitea,
            "codeberg.org".into(),
            "me/app".into(),
            "dev".into(),
            "CONTRIBUTING.md".into()
        )
    );
    assert_eq!(
        link("https://git.example:3000/me/app/src/tag/v2/docs/x.md")
            .1
            .as_str(),
        "git.example:3000"
    );
}

#[test]
fn raw_links_are_read_and_name_their_host() {
    assert_eq!(
        link("https://raw.githubusercontent.com/acme/app/main/docs/REVIEW.md"),
        (
            LinkKind::Github,
            "github.com".into(),
            "acme/app".into(),
            "main".into(),
            "docs/REVIEW.md".into()
        )
    );
    assert_eq!(
        link("https://raw.githubusercontent.com/acme/app/refs/heads/dev/a.md").3,
        "dev"
    );
    assert_eq!(
        link("https://gitlab.com/g/p/-/raw/main/a.md"),
        (
            LinkKind::Gitlab,
            "gitlab.com".into(),
            "g/p".into(),
            "main".into(),
            "a.md".into()
        )
    );
    assert_eq!(
        link("https://codeberg.org/me/app/raw/branch/main/a.md").0,
        LinkKind::Gitea
    );
    assert_eq!(
        link("https://ghe.corp.example/o/r/raw/main/a.md").1,
        "ghe.corp.example"
    );
}

#[test]
fn what_is_not_a_file_link_is_not_parsed() {
    for url in [
        "https://example.org/notes.md",
        "https://github.com/acme/app",
        "https://github.com/acme/app/blob/main",
        "https://github.com/acme/app/pull/3",
        "ftp://github.com/acme/app/blob/main/a.md",
        "docs/REVIEW.md",
        "",
    ] {
        assert!(parse_file_link(url).is_none(), "{url}");
    }
}

#[test]
fn a_link_has_a_raw_address() {
    let raw = |url: &str| parse_file_link(url).unwrap().raw_url();
    assert_eq!(
        raw("https://github.com/o/r/blob/main/a/b.md"),
        "https://raw.githubusercontent.com/o/r/main/a/b.md"
    );
    assert_eq!(
        raw("https://ghe.example/o/r/blob/main/a.md"),
        "https://ghe.example/o/r/raw/main/a.md"
    );
    assert_eq!(
        raw("https://gitlab.example/g/p/-/blob/main/a.md"),
        "https://gitlab.example/g/p/-/raw/main/a.md"
    );
    assert_eq!(
        raw("https://codeberg.org/o/r/src/tag/v1/a.md"),
        "https://codeberg.org/o/r/raw/tag/v1/a.md"
    );
}

// -- gathering and caps ----------------------------------------------------

#[derive(Default)]
struct Fake {
    repo: BTreeMap<String, Vec<u8>>,
    links: BTreeMap<String, Vec<u8>>,
    asked: Mutex<Vec<String>>,
}

impl Fake {
    fn with(mut self, path: &str, text: &str) -> Self {
        self.repo.insert(path.into(), text.as_bytes().to_vec());
        self
    }
}

#[async_trait]
impl ContextSource for Fake {
    async fn repo_file(&self, path: &str) -> Result<Option<Vec<u8>>, String> {
        self.asked.lock().unwrap().push(format!("repo:{path}"));
        Ok(self.repo.get(path).cloned())
    }
    async fn link_file(&self, url: &str) -> Result<Option<Vec<u8>>, String> {
        self.asked.lock().unwrap().push(format!("link:{url}"));
        Ok(self.links.get(url).cloned())
    }
}

fn ctx(claude: bool, agents: bool, extra: &str) -> EffectiveReviewContext {
    EffectiveReviewContext {
        run_in_repository: RunInRepo::Profile,
        include_claude_md: claude,
        include_agents_md: agents,
        extra_file: extra.into(),
    }
}

#[test]
fn only_the_asked_files_are_read_and_a_missing_one_is_skipped() {
    let source = Fake::default().with("AGENTS.md", "be kind");
    let got = block_on(gather_report(&ctx(true, true, ""), &source));
    assert_eq!(
        got.files,
        [ContextFile {
            label: "AGENTS.md".into(),
            text: "be kind".into(),
            cut: false
        }]
    );
    assert!(got.problems.is_empty(), "a missing CLAUDE.md is no problem");
    let off = Fake::default()
        .with("CLAUDE.md", "x")
        .with("AGENTS.md", "y");
    assert!(block_on(gather(&ctx(false, false, ""), &off)).is_empty());
    assert!(off.asked.lock().unwrap().is_empty(), "nothing is read");
}

#[test]
fn a_file_over_8_kb_is_cut_with_a_visible_note() {
    let source = Fake::default().with("CLAUDE.md", &"a".repeat(20_000));
    let got = block_on(gather(&ctx(true, false, ""), &source));
    assert_eq!(got.len(), 1);
    assert!(got[0].cut);
    assert!(got[0].text.starts_with(&"a".repeat(MAX_FILE_BYTES)));
    assert!(
        got[0]
            .text
            .ends_with("[CLAUDE.md cut: showing 8192 of 20000 bytes]"),
        "{}",
        &got[0].text[MAX_FILE_BYTES..]
    );
}

#[test]
fn the_cut_never_splits_a_character() {
    // 3-byte characters: 8192 is not a multiple of 3.
    let source = Fake::default().with("CLAUDE.md", &"€".repeat(5_000));
    let got = block_on(gather(&ctx(true, false, ""), &source));
    assert!(got[0].cut);
    assert!(got[0].text.starts_with(&"€".repeat(2_730)));
}

#[test]
fn the_files_together_stop_at_16_kb() {
    let source = Fake::default()
        .with("CLAUDE.md", &"c".repeat(12_000))
        .with("AGENTS.md", &"a".repeat(12_000))
        .with("docs/extra.md", "never given");
    let got = block_on(gather_report(&ctx(true, true, "docs/extra.md"), &source));
    assert_eq!(got.files.len(), 2);
    assert!(got.files[0].cut, "8 KB of the first");
    assert!(got.files[1].cut, "8 KB of the second, the total is spent");
    let given: usize = got
        .files
        .iter()
        .map(|f| f.text.find("\n[").unwrap_or(f.text.len()))
        .sum();
    assert_eq!(given, MAX_TOTAL_BYTES);
    assert_eq!(got.problems.len(), 1, "{:?}", got.problems);
    assert!(got.problems[0].starts_with("docs/extra.md: left out"));
}

#[test]
fn the_total_cap_shortens_the_file_that_crosses_it() {
    let source = Fake::default()
        .with("CLAUDE.md", &"c".repeat(7_000))
        .with("AGENTS.md", &"a".repeat(7_000))
        .with("docs/extra.md", &"e".repeat(7_000));
    let got = block_on(gather(&ctx(true, true, "docs/extra.md"), &source));
    assert_eq!(got.len(), 3);
    assert!(!got[0].cut && !got[1].cut);
    assert!(got[2].cut, "16384 - 14000 = 2384 bytes left");
    assert!(got[2].text.starts_with(&"e".repeat(2_384)));
    assert!(!got[2].text.starts_with(&"e".repeat(2_385)));
}

#[test]
fn a_relative_extra_file_is_read_from_the_repository_and_cannot_leave_it() {
    let source = Fake::default().with("docs/REVIEW.md", "rules");
    let got = block_on(gather(&ctx(false, false, "./docs/REVIEW.md"), &source));
    assert_eq!(got[0].label, "./docs/REVIEW.md");
    assert_eq!(
        source.asked.lock().unwrap().as_slice(),
        ["repo:docs/REVIEW.md"]
    );

    let report = block_on(gather_report(&ctx(false, false, "../secret.md"), &source));
    assert!(report.files.is_empty());
    assert_eq!(report.problems.len(), 1);
    assert!(report.problems[0].contains("inside the repository"));
    assert_eq!(source.asked.lock().unwrap().len(), 1, "nothing was read");
}

#[test]
fn a_missing_extra_file_is_reported() {
    let source = Fake::default();
    let report = block_on(gather_report(&ctx(false, false, "docs/none.md"), &source));
    assert!(report.files.is_empty());
    assert_eq!(report.problems, ["docs/none.md: not found"]);
}

#[test]
fn an_absolute_path_is_read_from_disk_and_a_link_goes_to_the_link_reader() {
    let dir = tempfile::tempdir().unwrap();
    let file = dir.path().join("team.md");
    std::fs::write(&file, "team rules").unwrap();
    let source = Fake::default();
    let got = block_on(gather(&ctx(false, false, file.to_str().unwrap()), &source));
    assert_eq!(got[0].text, "team rules");
    assert!(
        source.asked.lock().unwrap().is_empty(),
        "disk, not the repo"
    );

    let mut with_link = Fake::default();
    with_link
        .links
        .insert("https://example.org/r.md".into(), b"linked".to_vec());
    let got = block_on(gather(
        &ctx(false, false, "https://example.org/r.md"),
        &with_link,
    ));
    assert_eq!(got[0].text, "linked");
    assert_eq!(got[0].label, "https://example.org/r.md");
}

// -- links go through the profile matching their host ----------------------

fn profiles() -> Vec<Integration> {
    vec![
        Integration::new("gh", IntegrationKind::Github, "https://github.com"),
        Integration::new(
            "corp-gl",
            IntegrationKind::GitlabSelfManaged,
            "https://gitlab.corp.example",
        ),
        {
            let mut off = Integration::new(
                "off-gitea",
                IntegrationKind::Gitea,
                "https://git.off.example",
            );
            off.enabled = false;
            off
        },
    ]
}

#[derive(Default)]
struct Reader(Mutex<Vec<String>>);

#[async_trait]
impl ProfileReader for Reader {
    async fn read(
        &self,
        integration: &Integration,
        project: &str,
        path: &str,
        rev: &str,
    ) -> Result<Option<Vec<u8>>, String> {
        self.0
            .lock()
            .unwrap()
            .push(format!("{}|{project}|{path}|{rev}", integration.id));
        Ok(Some(b"from the profile".to_vec()))
    }
}

#[test]
fn a_link_is_read_through_the_profile_of_its_host() {
    let integrations = profiles();
    let reader = Reader::default();
    let transport = RecordedTransport::new();
    for url in [
        "https://gitlab.corp.example/g/p/-/blob/main/docs/a.md",
        "https://github.com/o/r/blob/dev/b.md",
        "https://raw.githubusercontent.com/o/r/dev/b.md",
    ] {
        let got = block_on(fetch_link(url, &integrations, &reader, &transport)).unwrap();
        assert_eq!(got.as_deref(), Some(&b"from the profile"[..]), "{url}");
    }
    assert_eq!(
        reader.0.lock().unwrap().as_slice(),
        [
            "corp-gl|g/p|docs/a.md|main",
            "gh|o/r|b.md|dev",
            "gh|o/r|b.md|dev"
        ]
    );
    assert!(transport.requests().is_empty(), "no anonymous request");
}

#[test]
fn a_link_of_an_unknown_or_disabled_host_is_read_anonymously() {
    let integrations = profiles();
    let reader = Reader::default();
    let transport = RecordedTransport::new()
        .on("raw/branch/main/a.md", 200, "public rules")
        .on("example.org/notes.md", 200, "plain");
    // A Gitea host whose profile is disabled, a host with no profile, and a
    // page that is no file link at all.
    let got = block_on(fetch_link(
        "https://git.off.example/o/r/src/branch/main/a.md",
        &integrations,
        &reader,
        &transport,
    ))
    .unwrap();
    assert_eq!(got.as_deref(), Some(&b"public rules"[..]));
    let got = block_on(fetch_link(
        "https://example.org/notes.md",
        &integrations,
        &reader,
        &transport,
    ))
    .unwrap();
    assert_eq!(got.as_deref(), Some(&b"plain"[..]));
    // A gitlab-style link on the GitHub host does not use the GitHub profile.
    let none = block_on(fetch_link(
        "https://github.com/g/p/-/blob/main/a.md",
        &integrations,
        &reader,
        &transport,
    ));
    assert!(none.is_err() || none.unwrap().is_none());
    assert!(reader.0.lock().unwrap().is_empty(), "no profile was used");
    let urls: Vec<String> = transport.requests().into_iter().map(|r| r.url).collect();
    assert_eq!(urls[0], "https://git.off.example/o/r/raw/branch/main/a.md");
    assert_eq!(urls[1], "https://example.org/notes.md");
    assert!(
        transport.requests().iter().all(|r| !r
            .headers
            .iter()
            .any(|(k, _)| k.eq_ignore_ascii_case("authorization"))),
        "anonymous means no token"
    );
}

#[test]
fn an_anonymous_404_is_a_missing_file_and_other_errors_are_errors() {
    let transport =
        RecordedTransport::new()
            .on("/gone.md", 404, "nope")
            .on("/denied.md", 403, "no");
    assert_eq!(
        block_on(fetch_anonymous(&transport, "https://x.example/gone.md")).unwrap(),
        None
    );
    assert!(block_on(fetch_anonymous(&transport, "https://x.example/denied.md")).is_err());
}

#[test]
fn the_description_names_the_files_and_the_run_mode() {
    let text = ctx(true, false, "docs/R.md").describe();
    assert!(text.contains("CLAUDE.md, docs/R.md"), "{text}");
    assert!(!text.contains("AGENTS.md"), "{text}");
    assert!(text.contains("where its AI profile says"), "{text}");
    assert!(ctx(false, false, "")
        .describe()
        .contains("no project conventions"));
}
