//! References in commit messages and branch names (plan D7).
//!
//! Pure: what a token means depends on the repository's linked ISSUES and
//! PULLS families, handed in as a [`RefContext`]. Unlinked means nothing is
//! a link. The TUI and the CLI both scan through here, so they cannot
//! disagree about what `ABC-12` or `#7` refers to.
//!
//! Offsets are char offsets into the scanned text, `[start, end)`.

use gitpoulpi_config::ApiFamily;

use crate::spec::spec_for;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RefKind {
    Issue,
    PullRequest,
}

/// One reference found in a message or branch name.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FoundRef {
    pub start: usize,
    pub end: usize,
    /// The text as written (`AB#7`, `#7`, `ABC-12`).
    pub text: String,
    pub kind: RefKind,
    /// The canonical spelling a human would type (`#7`, `AB#7`, `!4`, `ABC-12`).
    pub reference: String,
    pub url: Option<String>,
}

/// Trackers whose items are `KEY-N` (key of a project the user linked).
trait KeyTracker {
    fn is_key_tracker(&self) -> bool;
}

impl KeyTracker for ApiFamily {
    fn is_key_tracker(&self) -> bool {
        matches!(
            self,
            ApiFamily::Jira | ApiFamily::Linear | ApiFamily::YouTrack
        )
    }
}

/// A Trello card known from the cached snapshot.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KnownCard {
    pub id_short: u64,
    pub url: String,
}

/// The linked issue tracker.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IssueLinker {
    pub family: ApiFamily,
    pub web_base: String,
    /// The linked project (Jira, Linear, YouTrack: its key).
    pub project: String,
    /// Jira, Linear, YouTrack: further project keys seen in the profile's cached project list.
    pub project_keys: Vec<String>,
    /// Trello: the cached cards (for `#N` and `N-slug`).
    pub cards: Vec<KnownCard>,
}

/// The linked pull-request source.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PullLinker {
    pub family: ApiFamily,
    pub web_base: String,
    pub project: String,
}

/// From the repository's two links; both `None` = nothing is a link.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct RefContext {
    pub issues: Option<IssueLinker>,
    pub pulls: Option<PullLinker>,
}

/// Digits of a number token: 1 to 9 of them.
const MAX_DIGITS: usize = 9;

fn is_word(c: char) -> bool {
    c.is_alphanumeric() || c == '_'
}

/// The run of ASCII digits at `from`, as `(end, value)`.
fn digits_at(chars: &[char], from: usize) -> Option<(usize, u64)> {
    let mut j = from;
    while j < chars.len() && chars[j].is_ascii_digit() {
        j += 1;
    }
    if j == from || j - from > MAX_DIGITS {
        return None;
    }
    let n = chars[from..j].iter().collect::<String>().parse().ok()?;
    Some((j, n))
}

fn slice(chars: &[char], a: usize, b: usize) -> String {
    chars[a..b].iter().collect()
}

impl IssueLinker {
    fn jira_key_known(&self, key: &str) -> bool {
        key == self.project || self.project_keys.iter().any(|k| k == key)
    }

    fn card_url(&self, id_short: u64) -> Option<&str> {
        self.cards
            .iter()
            .find(|c| c.id_short == id_short)
            .map(|c| c.url.as_str())
    }

    /// Does this tracker read a bare `#N` as one of its items?
    fn takes_hash(&self, n: u64) -> bool {
        match self.family {
            ApiFamily::Github
            | ApiFamily::Gitlab
            | ApiFamily::AzureDevops
            | ApiFamily::Gitea
            | ApiFamily::Redmine
            | ApiFamily::LocalMarkdown => true,
            ApiFamily::Trello => self.card_url(n).is_some(),
            _ => false,
        }
    }
}

struct Matcher<'a> {
    ctx: &'a RefContext,
}

impl Matcher<'_> {
    fn azure_issues(&self) -> Option<&IssueLinker> {
        self.ctx
            .issues
            .as_ref()
            .filter(|i| i.family == ApiFamily::AzureDevops)
    }

    fn issue_number_ref(&self, issues: &IssueLinker, n: u64) -> (String, Option<String>) {
        match issues.family {
            ApiFamily::AzureDevops => {
                // `#N` is ambiguous when the pull requests also use `#N`.
                let pulls_azure = self
                    .ctx
                    .pulls
                    .as_ref()
                    .is_some_and(|p| p.family == ApiFamily::AzureDevops);
                let reference = if pulls_azure {
                    format!("#{n}")
                } else {
                    format!("AB#{n}")
                };
                (reference, self.issue_url(issues, n))
            }
            ApiFamily::Trello => {
                let url = issues.card_url(n).map(str::to_string);
                (url.clone().unwrap_or_default(), url)
            }
            _ => (format!("#{n}"), self.issue_url(issues, n)),
        }
    }

    fn issue_url(&self, issues: &IssueLinker, n: u64) -> Option<String> {
        (spec_for(issues.family).issue_web_url)(&issues.web_base, &issues.project, &n.to_string())
    }

    /// A reference starting at `i`, as `(end, found)`.
    fn at(&self, chars: &[char], i: usize) -> Option<(usize, FoundRef)> {
        let c = chars[i];
        let prev = i.checked_sub(1).map(|p| chars[p]);
        let boundary = prev.is_none_or(|p| !is_word(p) && p != '&');
        let issues = self.ctx.issues.as_ref();
        let make = |end: usize, kind, reference: String, url: Option<String>| FoundRef {
            start: i,
            end,
            text: slice(chars, i, end),
            kind,
            reference,
            url,
        };

        // Trello card URL.
        if c == 'h' && boundary && issues.is_some_and(|t| t.family == ApiFamily::Trello) {
            {
                let rest = slice(chars, i, chars.len().min(i + 25));
                let prefix = "https://trello.com/c/";
                if rest.starts_with(prefix) {
                    let mut j = i + prefix.len();
                    let key_start = j;
                    while j < chars.len() && chars[j].is_ascii_alphanumeric() {
                        j += 1;
                    }
                    if j > key_start {
                        let short = slice(chars, key_start, j);
                        // Optional `/12-slug` tail.
                        let mut end = j;
                        if chars.get(j) == Some(&'/') {
                            let mut k = j + 1;
                            while k < chars.len()
                                && (chars[k].is_ascii_alphanumeric() || chars[k] == '-')
                            {
                                k += 1;
                            }
                            if k > j + 1 {
                                end = k;
                            }
                        }
                        let url = format!("https://trello.com/c/{short}");
                        return Some((end, make(end, RefKind::Issue, url.clone(), Some(url))));
                    }
                }
            }
        }

        // Jira, Linear and YouTrack `KEY-N`.
        if c.is_ascii_uppercase() && prev.is_none_or(|p| !is_word(p) && p != '-') {
            if let Some(j) = issues.filter(|j| j.family.is_key_tracker()) {
                let mut k = i;
                while k < chars.len()
                    && (chars[k].is_ascii_uppercase()
                        || chars[k].is_ascii_digit()
                        || chars[k] == '_')
                {
                    k += 1;
                }
                if chars.get(k) == Some(&'-') {
                    let key = slice(chars, i, k);
                    if let Some((end, _)) = digits_at(chars, k + 1) {
                        if j.jira_key_known(&key) && chars.get(end).is_none_or(|n| !is_word(*n)) {
                            let reference = slice(chars, i, end);
                            let url = (spec_for(j.family).issue_web_url)(
                                &j.web_base,
                                &j.project,
                                &reference,
                            );
                            return Some((end, make(end, RefKind::Issue, reference, url)));
                        }
                    }
                }
            }
        }

        // Azure `AB#N`.
        if c == 'A' && boundary && chars.get(i + 1) == Some(&'B') && chars.get(i + 2) == Some(&'#')
        {
            if let Some(az) = self.azure_issues() {
                if let Some((end, n)) = digits_at(chars, i + 3) {
                    if chars.get(end).is_none_or(|n| !is_word(*n)) {
                        let (reference, url) = self.issue_number_ref(az, n);
                        return Some((end, make(end, RefKind::Issue, reference, url)));
                    }
                }
            }
        }

        // `#N` and `!N`.
        if (c == '#' || c == '!') && boundary {
            let (end, n) = digits_at(chars, i + 1)?;
            if chars.get(end).is_some_and(|n| is_word(*n)) {
                return None;
            }
            if c == '#' {
                if let Some(is) = issues.filter(|is| is.takes_hash(n)) {
                    let (reference, url) = self.issue_number_ref(is, n);
                    return Some((end, make(end, RefKind::Issue, reference, url)));
                }
            }
            if let Some(p) = self.ctx.pulls.as_ref() {
                if spec_for(p.family).pr_sigil == c {
                    let url = (spec_for(p.family).pr_web_url)(&p.web_base, &p.project, n);
                    return Some((end, make(end, RefKind::PullRequest, format!("{c}{n}"), url)));
                }
            }
        }
        None
    }
}

/// Every reference in a commit message, in order, that the linked sources
/// give a meaning to.
pub fn scan_message(text: &str, ctx: &RefContext) -> Vec<FoundRef> {
    if ctx.issues.is_none() && ctx.pulls.is_none() {
        return Vec::new();
    }
    let chars: Vec<char> = text.chars().collect();
    let matcher = Matcher { ctx };
    let mut out = Vec::new();
    let mut i = 0;
    while i < chars.len() {
        match matcher.at(&chars, i) {
            Some((end, found)) => {
                out.push(found);
                i = end;
            }
            None => i += 1,
        }
    }
    out
}

/// The issue a branch name names: `ABC-123-x` / `feature/ABC-123` (Jira),
/// `123-slug` (the other trackers; Trello through its cached cards).
pub fn from_branch(branch: &str, ctx: &RefContext) -> Option<FoundRef> {
    let issues = ctx.issues.as_ref()?;
    let chars: Vec<char> = branch.chars().collect();
    let matcher = Matcher { ctx };
    if issues.family.is_key_tracker() {
        let mut i = 0;
        while i < chars.len() {
            if let Some((_, found)) = matcher.at(&chars, i) {
                return Some(found);
            }
            i += 1;
        }
        return None;
    }
    let (end, n) = digits_at(&chars, 0)?;
    if chars.get(end) != Some(&'-') || chars.len() == end + 1 {
        return None;
    }
    if issues.family == ApiFamily::Trello && issues.card_url(n).is_none() {
        return None;
    }
    if !matches!(
        issues.family,
        ApiFamily::Github
            | ApiFamily::Gitlab
            | ApiFamily::AzureDevops
            | ApiFamily::Trello
            | ApiFamily::Gitea
            | ApiFamily::Redmine
            | ApiFamily::LocalMarkdown
    ) {
        return None;
    }
    let (reference, url) = matcher.issue_number_ref(issues, n);
    Some(FoundRef {
        start: 0,
        end,
        text: slice(&chars, 0, end),
        kind: RefKind::Issue,
        reference,
        url,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn issues(family: ApiFamily, base: &str, project: &str) -> IssueLinker {
        IssueLinker {
            family,
            web_base: base.into(),
            project: project.into(),
            project_keys: Vec::new(),
            cards: Vec::new(),
        }
    }

    fn pulls(family: ApiFamily, base: &str, project: &str) -> PullLinker {
        PullLinker {
            family,
            web_base: base.into(),
            project: project.into(),
        }
    }

    fn gl() -> RefContext {
        RefContext {
            issues: Some(issues(ApiFamily::Gitlab, "https://gl", "g/p")),
            pulls: Some(pulls(ApiFamily::Gitlab, "https://gl", "g/p")),
        }
    }

    fn jira() -> RefContext {
        let mut i = issues(ApiFamily::Jira, "https://acme.atlassian.net", "ABC");
        i.project_keys = vec!["ABC".into(), "OPS".into()];
        RefContext {
            issues: Some(i),
            pulls: None,
        }
    }

    fn azure(pull: Option<PullLinker>) -> RefContext {
        RefContext {
            issues: Some(issues(
                ApiFamily::AzureDevops,
                "https://dev.azure.com/o",
                "P",
            )),
            pulls: pull,
        }
    }

    fn trello() -> RefContext {
        let mut i = issues(ApiFamily::Trello, "https://trello.com", "board");
        i.cards = vec![KnownCard {
            id_short: 7,
            url: "https://trello.com/c/Ab12CdEf".into(),
        }];
        RefContext {
            issues: Some(i),
            pulls: None,
        }
    }

    /// `(text, reference, url)` of every found ref.
    fn found(text: &str, ctx: &RefContext) -> Vec<(String, String, Option<String>)> {
        scan_message(text, ctx)
            .into_iter()
            .map(|f| (f.text, f.reference, f.url))
            .collect()
    }

    fn texts(text: &str, ctx: &RefContext) -> Vec<String> {
        scan_message(text, ctx)
            .into_iter()
            .map(|f| f.text)
            .collect()
    }

    #[test]
    fn github_and_gitlab_hash_numbers() {
        let ctx = RefContext {
            issues: Some(issues(ApiFamily::Github, "https://github.com", "o/r")),
            pulls: None,
        };
        let f = scan_message("fix (#12), see #7.", &ctx);
        assert_eq!(f.len(), 2);
        assert_eq!(f[0].text, "#12");
        assert_eq!(f[0].kind, RefKind::Issue);
        assert_eq!(
            f[0].url.as_deref(),
            Some("https://github.com/o/r/issues/12")
        );
        assert_eq!((f[0].start, f[0].end), (5, 8));
    }

    #[test]
    fn gitlab_issue_and_merge_request() {
        let f = found("Closes #4242, see also !4343.", &gl());
        assert_eq!(
            f,
            vec![
                (
                    "#4242".to_string(),
                    "#4242".to_string(),
                    Some("https://gl/g/p/-/issues/4242".to_string())
                ),
                (
                    "!4343".to_string(),
                    "!4343".to_string(),
                    Some("https://gl/g/p/-/merge_requests/4343".to_string())
                ),
            ]
        );
        assert_eq!(scan_message("!1", &gl())[0].kind, RefKind::PullRequest);
    }

    #[test]
    fn github_pulls_alone_claim_hash() {
        let ctx = RefContext {
            issues: None,
            pulls: Some(pulls(ApiFamily::Github, "https://github.com", "o/r")),
        };
        let f = scan_message("merge #5 not !5", &ctx);
        assert_eq!(f.len(), 1);
        assert_eq!(f[0].kind, RefKind::PullRequest);
        assert_eq!(f[0].url.as_deref(), Some("https://github.com/o/r/pull/5"));
    }

    #[test]
    fn issues_win_over_pulls_on_hash() {
        let ctx = RefContext {
            issues: Some(issues(ApiFamily::Github, "https://github.com", "o/r")),
            pulls: Some(pulls(ApiFamily::Github, "https://github.com", "o/r")),
        };
        let f = scan_message("#5", &ctx);
        assert_eq!(f[0].kind, RefKind::Issue);
        assert_eq!(f[0].url.as_deref(), Some("https://github.com/o/r/issues/5"));
    }

    #[cfg(feature = "jira")]
    #[test]
    fn jira_keys_only_for_known_projects() {
        let f = found("Fixes ABC-12, see UTF-8, OPS-3.", &jira());
        assert_eq!(
            f,
            vec![
                (
                    "ABC-12".to_string(),
                    "ABC-12".to_string(),
                    Some("https://acme.atlassian.net/browse/ABC-12".to_string())
                ),
                (
                    "OPS-3".to_string(),
                    "OPS-3".to_string(),
                    Some("https://acme.atlassian.net/browse/OPS-3".to_string())
                ),
            ]
        );
    }

    #[test]
    fn jira_negatives() {
        for text in [
            "UTF-8", "SHA-256", "ISO-8601", "abc-12", "XABC-12", "ABC-12x", "ABC-", "ABC_12", "#12",
        ] {
            assert!(texts(text, &jira()).is_empty(), "{text}");
        }
        // The linked key counts even without a cached project list.
        let mut ctx = jira();
        ctx.issues.as_mut().unwrap().project_keys.clear();
        assert_eq!(texts("ABC-1 OPS-2", &ctx), vec!["ABC-1"]);
    }

    #[cfg(feature = "gitea")]
    #[test]
    fn gitea_hash_numbers_link_to_issues_and_pulls_and_branches_start_with_the_number() {
        let ctx = RefContext {
            issues: Some(issues(ApiFamily::Gitea, "https://codeberg.org", "o/r")),
            pulls: Some(pulls(ApiFamily::Gitea, "https://codeberg.org", "o/r")),
        };
        let f = scan_message("fix #12, see #7.", &ctx);
        assert_eq!(f.len(), 2);
        // The issue tracker wins a bare `#N`, as on GitHub (the same URL
        // works for a pull request, Gitea redirects).
        assert_eq!(
            f[0].url.as_deref(),
            Some("https://codeberg.org/o/r/issues/12")
        );
        let branch = from_branch("12-fix-login", &ctx).expect("a branch reference");
        assert_eq!(
            branch.url.as_deref(),
            Some("https://codeberg.org/o/r/issues/12")
        );
    }

    #[test]
    fn hash_negatives_for_every_hash_family() {
        for text in [
            "abc#1",
            "&#39;",
            "#1a2b3c",
            "color #fff",
            "# 12",
            "#",
            "#12abc",
        ] {
            assert!(texts(text, &gl()).is_empty(), "{text}");
        }
        assert!(texts("wow! !important", &gl()).is_empty());
    }

    #[cfg(feature = "azure-devops")]
    #[test]
    fn azure_boards_hash_and_ab_hash() {
        let ctx = azure(None);
        let f = scan_message("Fixes AB#7 and #8", &ctx);
        assert_eq!(f.len(), 2);
        assert_eq!(f[0].text, "AB#7");
        assert_eq!((f[0].start, f[0].end), (6, 10));
        assert_eq!(
            f[0].url.as_deref(),
            Some("https://dev.azure.com/o/P/_workitems/edit/7")
        );
        assert_eq!(f[1].text, "#8");
        // Not glued to a word.
        assert!(texts("XAB#7 aAB#7", &ctx).is_empty());
    }

    #[test]
    fn azure_reference_spelling_depends_on_the_pulls_source() {
        // Azure Repos pulls use `!N`, so `#N` is unambiguous.
        let own = azure(Some(pulls(
            ApiFamily::AzureDevops,
            "https://dev.azure.com/o",
            "P/R",
        )));
        assert_eq!(scan_message("AB#7", &own)[0].reference, "#7");
        // A pulls source elsewhere (GitHub `#N`) makes `AB#N` the safe spelling.
        let other = azure(Some(pulls(ApiFamily::Github, "https://github.com", "o/r")));
        let f = scan_message("#7", &other);
        assert_eq!(f[0].reference, "AB#7");
        assert_eq!(f[0].kind, RefKind::Issue);
        assert_eq!(scan_message("#7", &azure(None))[0].reference, "AB#7");
    }

    #[test]
    fn ab_hash_is_for_azure_issues_only() {
        assert!(texts("AB#7", &gl()).is_empty());
        assert!(texts("AB#7", &jira()).is_empty());
        assert!(texts("AB#7", &trello()).is_empty());
        let github = RefContext {
            issues: Some(issues(ApiFamily::Github, "https://github.com", "o/r")),
            pulls: None,
        };
        assert!(texts("AB#7", &github).is_empty());
    }

    #[cfg(feature = "azure-devops")]
    #[test]
    fn azure_pulls_use_bang() {
        let ctx = RefContext {
            issues: None,
            pulls: Some(pulls(
                ApiFamily::AzureDevops,
                "https://dev.azure.com/o",
                "P/R",
            )),
        };
        let f = scan_message("see !12 and #3", &ctx);
        assert_eq!(f.len(), 1);
        assert_eq!(
            f[0].url.as_deref(),
            Some("https://dev.azure.com/o/P/_git/R/pullrequest/12")
        );
    }

    #[cfg(feature = "bitbucket")]
    #[test]
    fn bitbucket_pulls_use_hash() {
        let ctx = RefContext {
            issues: None,
            pulls: Some(pulls(ApiFamily::Bitbucket, "https://bitbucket.org", "w/r")),
        };
        let f = scan_message("PR #4 not !4", &ctx);
        assert_eq!(f.len(), 1);
        assert_eq!(
            f[0].url.as_deref(),
            Some("https://bitbucket.org/w/r/pull-requests/4")
        );
    }

    #[test]
    fn jira_issues_leave_hash_to_pulls() {
        let mut ctx = jira();
        ctx.pulls = Some(pulls(ApiFamily::Github, "https://github.com", "o/r"));
        let f = scan_message("ABC-1 via #9", &ctx);
        assert_eq!(f.len(), 2);
        assert_eq!(f[1].kind, RefKind::PullRequest);
    }

    #[test]
    fn trello_urls_and_known_hash_numbers() {
        let ctx = trello();
        let f = scan_message(
            "see https://trello.com/c/Zz9Yy8Xx/12-some-slug. and #7, not #8",
            &ctx,
        );
        assert_eq!(f.len(), 2);
        assert_eq!(f[0].text, "https://trello.com/c/Zz9Yy8Xx/12-some-slug");
        assert_eq!(f[0].url.as_deref(), Some("https://trello.com/c/Zz9Yy8Xx"));
        assert_eq!(f[1].text, "#7");
        assert_eq!(f[1].url.as_deref(), Some("https://trello.com/c/Ab12CdEf"));
    }

    #[test]
    fn unlinked_is_plain_text() {
        let ctx = RefContext::default();
        assert!(scan_message("#1 !2 ABC-3 AB#4", &ctx).is_empty());
        assert_eq!(from_branch("123-x", &ctx), None);
    }

    #[test]
    fn offsets_are_char_offsets() {
        let f = scan_message("é #5", &gl());
        assert_eq!((f[0].start, f[0].end), (2, 4));
    }

    #[test]
    fn branch_forms() {
        let b = |name: &str, ctx: &RefContext| from_branch(name, ctx).map(|f| f.reference);
        assert_eq!(b("ABC-123-x", &jira()).as_deref(), Some("ABC-123"));
        assert_eq!(b("feature/ABC-123", &jira()).as_deref(), Some("ABC-123"));
        assert_eq!(
            b("feature/ABC-123-fix-login", &jira()).as_deref(),
            Some("ABC-123")
        );
        assert_eq!(b("main", &jira()), None);
        assert_eq!(b("utf-8-fix", &jira()), None);
        assert_eq!(b("123-slug", &gl()).as_deref(), Some("#123"));
        let f = from_branch("123-slug", &gl()).unwrap();
        assert_eq!(f.url.as_deref(), Some("https://gl/g/p/-/issues/123"));
        assert_eq!((f.start, f.end), (0, 3));
        assert_eq!(b("123", &gl()), None);
        assert_eq!(b("fix-123", &gl()), None);
        // No `AB#` in a branch name; Azure uses N-slug.
        assert_eq!(b("123-slug", &azure(None)).as_deref(), Some("AB#123"));
        assert_eq!(b("AB#123", &azure(None)), None);
        // Trello: idShort through the cached cards.
        assert_eq!(
            b("7-fix-login", &trello()).as_deref(),
            Some("https://trello.com/c/Ab12CdEf")
        );
        assert_eq!(b("8-fix-login", &trello()), None);
    }
}
