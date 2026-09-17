//! The provider's own "new issue" / "new pull request" web forms, as URLs
//! prefilled with what the user typed. No token, no network: a URL.
//!
//! Formats (GitHub docs "Creating an issue" / "Using query parameters to
//! create a pull request"; GitLab's `issue[...]` / `merge_request[...]`
//! form parameters):
//! - GitHub issue: `/{project}/issues/new?title=&body=&labels=a,b`
//! - GitHub PR: `/{project}/compare/{base}...{head}?expand=1&title=&body=`
//!   (a fork's head is `owner:branch`); no query parameter marks a draft.
//! - GitLab issue: `/{project}/-/issues/new?issue[title]=&issue[description]=`
//! - GitLab MR: `/{project}/-/merge_requests/new?merge_request[source_branch]=...`
//!   Labels have no name parameter on GitLab (only ids), so they travel as a
//!   `/label ~"x"` quick action appended to the description; a draft is the
//!   `Draft: ` title prefix. A fork's MR opens on the fork (the source
//!   project), where the form's target-project picker takes over.

use crate::IntegrationKind;
use gitpoulpi_config::ApiFamily;

/// Longest URL produced; browsers and hosts refuse far less than ~8 kB
/// (GitHub answers 414 above it), so the description is cut to fit.
pub const MAX_URL_LEN: usize = 6000;

/// Appended to a description that had to be cut.
pub const TRUNCATION_NOTE: &str = "\n\n[...truncated: paste the rest]";

/// Percent-encodes a query component: everything but RFC 3986 unreserved.
pub fn percent_encode_component(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    for b in text.bytes() {
        match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'.' | b'_' | b'~' => {
                out.push(b as char)
            }
            _ => out.push_str(&format!("%{b:02X}")),
        }
    }
    out
}

/// `prefix` + the query built from `params` (already-encoded names, raw
/// values), the last parameter being `body_key` whose value is cut so the
/// whole URL stays under [`MAX_URL_LEN`].
pub(crate) fn finish(
    prefix: &str,
    params: &[(String, String)],
    body_key: &str,
    body: &str,
) -> String {
    let mut url = prefix.to_string();
    for (k, v) in params {
        url.push_str(&format!("{k}={}&", percent_encode_component(v)));
    }
    let used = url.len() + body_key.len() + 1;
    let mut text = body.to_string();
    let mut encoded = percent_encode_component(&text);
    if used + encoded.len() > MAX_URL_LEN {
        let note = percent_encode_component(TRUNCATION_NOTE);
        let room = MAX_URL_LEN.saturating_sub(used + note.len());
        let mut cut = String::new();
        let mut len = 0;
        for c in text.chars() {
            let e = percent_encode_component(&c.to_string()).len();
            if len + e > room {
                break;
            }
            len += e;
            cut.push(c);
        }
        cut.push_str(TRUNCATION_NOTE);
        text = cut;
        encoded = percent_encode_component(&text);
    }
    if encoded.is_empty() {
        url.pop(); // the trailing `&` (or `?` when nothing was added)
        return url;
    }
    url.push_str(&format!("{body_key}={encoded}"));
    url
}

pub(crate) fn tidy(url: String) -> String {
    url.trim_end_matches(['&', '?']).to_string()
}

pub(crate) fn gitlab_labels(labels: &[String]) -> String {
    let list: Vec<String> = labels
        .iter()
        .map(|l| l.trim())
        .filter(|l| !l.is_empty())
        .map(|l| format!("~\"{}\"", l.replace('"', "")))
        .collect();
    if list.is_empty() {
        String::new()
    } else {
        format!("/label {}", list.join(" "))
    }
}

/// The provider's "new issue" form for `project`, prefilled.
///
/// A family whose spec has no form of its own gets the GitLab shape, as it
/// did before the spec table (its slice replaces that).
pub fn new_issue_form_url(
    kind: IntegrationKind,
    web_base: &str,
    project: &str,
    title: &str,
    body: &str,
    labels: &[String],
) -> String {
    let new_issue = crate::spec::spec_for(kind.family()).new_issue_url;
    new_issue(web_base, project, title, body, labels)
        .or_else(|| {
            let gitlab = crate::spec::spec_for(ApiFamily::Gitlab).new_issue_url;
            gitlab(web_base, project, title, body, labels)
        })
        .unwrap_or_default()
}

/// What the "new pull / merge request" form is prefilled with.
#[derive(Debug, Clone, Default)]
pub struct PullRequestForm<'a> {
    /// The base (target) project.
    pub project: &'a str,
    /// The head's project when it is a fork.
    pub source_project: Option<&'a str>,
    pub source_branch: &'a str,
    pub target_branch: Option<&'a str>,
    pub title: &'a str,
    pub body: &'a str,
    pub draft: bool,
}

/// The provider's "new pull / merge request" form, prefilled.
///
/// A family whose spec has no form of its own gets the GitLab shape, as it
/// did before the spec table (its slice replaces that).
pub fn new_pull_request_form_url(
    kind: IntegrationKind,
    web_base: &str,
    form: &PullRequestForm<'_>,
) -> String {
    let spec = crate::spec::spec_for(kind.family());
    (spec.new_pr_form_url)(web_base, form)
        .or_else(|| (crate::spec::spec_for(ApiFamily::Gitlab).new_pr_form_url)(web_base, form))
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;
    use IntegrationKind as K;

    #[test]
    fn github_issue_with_labels_and_special_characters() {
        let url = new_issue_form_url(
            K::Github,
            "https://github.com",
            "me/thing",
            "It breaks & \"burns\"",
            "line 1\nline 2 \u{e9}\u{4e2d} #5 100%",
            &["bug".into(), " ".into(), "ui".into()],
        );
        assert_eq!(
            url,
            "https://github.com/me/thing/issues/new?title=It%20breaks%20%26%20%22burns%22&labels=bug%2Cui&body=line%201%0Aline%202%20%C3%A9%E4%B8%AD%20%235%20100%25"
        );
    }

    #[test]
    fn github_enterprise_host_and_empty_fields() {
        let url = new_issue_form_url(K::Github, "https://ghe.corp.example/", "o/r", "T", "", &[]);
        assert_eq!(url, "https://ghe.corp.example/o/r/issues/new?title=T");
        let none = new_issue_form_url(K::Github, "https://github.com", "o/r", "", "", &[]);
        assert_eq!(none, "https://github.com/o/r/issues/new");
    }

    #[test]
    fn gitlab_issue_com_and_self_managed_nested_groups() {
        let url = new_issue_form_url(
            K::Gitlab,
            "https://gitlab.com",
            "g/p",
            "A title",
            "desc\nmore",
            &["bug".into(), "two words".into()],
        );
        assert_eq!(
            url,
            "https://gitlab.com/g/p/-/issues/new?issue%5Btitle%5D=A%20title&issue%5Bdescription%5D=desc%0Amore%0A%0A%2Flabel%20~%22bug%22%20~%22two%20words%22"
        );
        let nested = new_issue_form_url(
            K::GitlabSelfManaged,
            "https://git.corp.example",
            "a/b/c/d",
            "T",
            "",
            &[],
        );
        assert_eq!(
            nested,
            "https://git.corp.example/a/b/c/d/-/issues/new?issue%5Btitle%5D=T"
        );
    }

    #[test]
    fn github_pull_request_plain_and_fork() {
        let mut form = PullRequestForm {
            project: "up/repo",
            source_branch: "feat/x y",
            target_branch: Some("main"),
            title: "Add x",
            body: "Why\n& how",
            ..Default::default()
        };
        assert_eq!(
            new_pull_request_form_url(K::Github, "https://github.com", &form),
            "https://github.com/up/repo/compare/main...feat/x%20y?expand=1&title=Add%20x&body=Why%0A%26%20how"
        );
        form.source_project = Some("me/repo");
        assert!(
            new_pull_request_form_url(K::Github, "https://github.com", &form)
                .starts_with("https://github.com/up/repo/compare/main...me:feat/x%20y?")
        );
        form.source_project = Some("me/renamed");
        assert!(
            new_pull_request_form_url(K::Github, "https://github.com", &form)
                .contains("...me:renamed:feat/x%20y?")
        );
    }

    #[test]
    fn gitlab_merge_request_nested_and_draft_and_fork() {
        let mut form = PullRequestForm {
            project: "g/sub/p",
            source_branch: "feature/login",
            target_branch: Some("main"),
            title: "Login",
            body: "b",
            draft: true,
            ..Default::default()
        };
        assert_eq!(
            new_pull_request_form_url(K::GitlabSelfManaged, "https://git.corp.example", &form),
            "https://git.corp.example/g/sub/p/-/merge_requests/new?merge_request%5Bsource_branch%5D=feature%2Flogin&merge_request%5Btarget_branch%5D=main&merge_request%5Btitle%5D=Draft%3A%20Login&merge_request%5Bdescription%5D=b"
        );
        form.source_project = Some("me/p");
        assert!(
            new_pull_request_form_url(K::Gitlab, "https://gitlab.com", &form)
                .starts_with("https://gitlab.com/me/p/-/merge_requests/new?")
        );
    }

    #[test]
    fn a_huge_body_is_cut_with_a_note_and_the_url_stays_short() {
        let body = "\u{e9}x ".repeat(5000);
        for kind in [K::Github, K::Gitlab] {
            let url = new_issue_form_url(kind, "https://h.example", "o/r", "T", &body, &[]);
            assert!(url.len() <= MAX_URL_LEN, "{}", url.len());
            assert!(url.ends_with(&percent_encode_component(TRUNCATION_NOTE)));
        }
    }
}
