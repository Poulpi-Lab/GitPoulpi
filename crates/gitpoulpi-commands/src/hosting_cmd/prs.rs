//! `gitpoulpi hosting` -- prs.

use gitpoulpi_config::ApiFamily;
use gitpoulpi_hosting::{pull_request_web_url, spec_for, IssueFilter, PrFilter, StateFilter};
use serde_json::{json, Value};

use crate::cli::{OpenKind, StateArg};
use crate::repo_cmd::Emitted;

use super::*;

pub(super) fn state_filter(state: StateArg) -> StateFilter {
    match state {
        StateArg::Open => StateFilter::Open,
        StateArg::Closed => StateFilter::Closed,
        StateArg::All => StateFilter::All,
    }
}

pub(super) fn merge(mut base: Value, key: &str, value: Value) -> Value {
    base[key] = value;
    base
}

pub(super) fn pr_line(pr: &gitpoulpi_hosting::PullRequestSummary) -> String {
    format!(
        "{:<6} {:<7} {}  ({} -> {}, {})",
        pr.iid_label,
        format!("{:?}", pr.state).to_lowercase(),
        pr.title,
        pr.source_branch,
        pr.target_branch,
        pr.author
    )
}

pub(super) async fn prs(
    env: &HostingEnv,
    target: &RepoTarget,
    filter: PrFilter,
) -> Result<Emitted, String> {
    let list = env
        .client(&target.integration, &target.integrations)?
        .list_pull_requests(&target.project, &filter)
        .await
        .map_err(|e| e.to_string())?;
    let human = list.iter().map(pr_line).collect::<Vec<_>>().join("\n");
    Ok(Emitted {
        exit_code: 0,
        json: merge(
            target_json(target),
            "pull_requests",
            serde_json::to_value(&list).map_err(|e| e.to_string())?,
        ),
        human,
    })
}

pub(super) async fn pr_for_branch(
    env: &HostingEnv,
    target: &RepoTarget,
    branch: &str,
) -> Result<Emitted, String> {
    let found = env
        .client(&target.integration, &target.integrations)?
        .pull_request_for_branch(&target.project, branch)
        .await
        .map_err(|e| e.to_string())?;
    let human = match &found {
        Some(pr) => format!("{}\n{}", pr_line(pr), pr.web_url),
        None => format!("no pull/merge request for `{branch}`"),
    };
    let json = merge(
        merge(target_json(target), "branch", json!(branch)),
        "pull_request",
        serde_json::to_value(&found).map_err(|e| e.to_string())?,
    );
    Ok(Emitted {
        json,
        human,
        exit_code: 0,
    })
}

pub(super) async fn issues(
    env: &HostingEnv,
    target: &RepoTarget,
    filter: IssueFilter,
) -> Result<Emitted, String> {
    let list = env
        .client(&target.integration, &target.integrations)?
        .list_issues(&target.project, &filter)
        .await
        .map_err(|e| e.to_string())?;
    let human = list
        .iter()
        .map(|i| {
            format!(
                "{:<8} {:<6} {}{}",
                if i.reference.is_empty() {
                    &i.iid_label
                } else {
                    &i.reference
                },
                format!("{:?}", i.state).to_lowercase(),
                i.title,
                if i.assignees.is_empty() {
                    String::new()
                } else {
                    format!("  (@{})", i.assignees.join(", @"))
                }
            )
        })
        .collect::<Vec<_>>()
        .join("\n");
    let mut rows = serde_json::to_value(&list).map_err(|e| e.to_string())?;
    // `kind` / `status` read better than the model's `*_label` names; the
    // originals stay for compatibility.
    for (row, issue) in rows.as_array_mut().into_iter().flatten().zip(&list) {
        row["kind"] = json!(issue.kind_label);
        row["status"] = json!(issue.status_label);
    }
    Ok(Emitted {
        exit_code: 0,
        json: merge(target_json(target), "issues", rows),
        human,
    })
}

/// `--body`, or the text of `--body-file` (`-`: stdin), or empty.
pub(super) fn read_issue_body(
    body: Option<String>,
    body_file: Option<std::path::PathBuf>,
) -> Result<String, String> {
    match (body, body_file) {
        (Some(body), _) => Ok(body),
        (None, Some(path)) if path.as_os_str() == "-" => {
            let mut text = String::new();
            std::io::Read::read_to_string(&mut std::io::stdin(), &mut text)
                .map_err(|e| format!("could not read the body from stdin: {e}"))?;
            Ok(text)
        }
        (None, Some(path)) => std::fs::read_to_string(&path)
            .map_err(|e| format!("could not read {}: {e}", path.display())),
        (None, None) => Ok(String::new()),
    }
}

#[allow(clippy::too_many_arguments)]
pub(super) async fn issue_create(
    env: &HostingEnv,
    target: &RepoTarget,
    title: String,
    body: String,
    labels: Vec<String>,
    choice: (Option<String>, Option<String>),
    web: bool,
    meta: NewMeta,
) -> Result<Emitted, String> {
    if title.trim().is_empty() {
        return Err("an issue needs a title (--title)".to_string());
    }
    let labels: Vec<String> = labels
        .into_iter()
        .map(|l| l.trim().to_string())
        .filter(|l| !l.is_empty())
        .collect();
    if web {
        let base = gitpoulpi_hosting::integration_web_base(&target.integration)
            .ok_or_else(|| "this profile has no web address".to_string())?;
        let url = gitpoulpi_hosting::new_issue_form_url(
            target.integration.kind,
            &base,
            &target.project,
            &title,
            &body,
            &labels,
        );
        return Ok(Emitted {
            exit_code: 0,
            json: merge(target_json(target), "url", json!(url)),
            human: url,
        });
    }
    let client = env.client(&target.integration, &target.integrations)?;
    let (issue_type, list_id) = match choice {
        (None, None) => (None, None),
        (wanted_type, wanted_list) => {
            let options = client
                .issue_create_options(&target.project)
                .await
                .map_err(|e| e.to_string())?;
            (
                wanted_type
                    .map(|t| resolve_choice(&options.types, &t, "--type", &target.integration))
                    .transpose()?,
                wanted_list
                    .map(|l| resolve_choice(&options.lists, &l, "--list", &target.integration))
                    .transpose()?,
            )
        }
    };
    let milestone = match &meta.milestone {
        Some(wanted) => Some(resolve_milestone(&*client, &target.project, wanted).await?),
        None => None,
    };
    let created = client
        .create_issue(&gitpoulpi_hosting::NewIssue {
            project: target.project.clone(),
            title,
            body,
            labels,
            issue_type,
            list_id,
            assignees: meta.assignees,
            milestone,
        })
        .await
        .map_err(|e| e.to_string())?;
    let reference = if created.reference.is_empty() {
        format!("#{}", created.number)
    } else {
        created.reference.clone()
    };
    let human = format!("opened issue {reference}: {}", created.web_url);
    Ok(Emitted {
        exit_code: 0,
        json: merge(
            merge(
                merge(target_json(target), "number", json!(created.number)),
                "reference",
                json!(reference),
            ),
            "web_url",
            json!(created.web_url),
        ),
        human,
    })
}

/// `--type` / `--list` to the provider's id: an exact id, else a label
/// ignoring case. A name the project does not offer lists what it does, so
/// nothing is posted with a guess.
fn resolve_choice(
    choices: &[gitpoulpi_hosting::Choice],
    wanted: &str,
    flag: &str,
    integration: &Integration,
) -> Result<String, String> {
    let wanted = wanted.trim();
    if choices.is_empty() {
        return Err(format!(
            "{flag} does not apply to {} (`{}`): it offers nothing to choose here",
            integration.kind.label(),
            integration.id
        ));
    }
    choices
        .iter()
        .find(|c| c.id == wanted)
        .or_else(|| {
            choices
                .iter()
                .find(|c| c.label.eq_ignore_ascii_case(wanted))
        })
        .map(|c| c.id.clone())
        .ok_or_else(|| {
            format!(
                "no {flag} `{wanted}` on this project; choose one of: {}",
                choices
                    .iter()
                    .map(|c| c.label.as_str())
                    .collect::<Vec<_>>()
                    .join(", ")
            )
        })
}

/// `hosting issue options`: what `issue create --type` / `--list` accept.
pub(super) async fn issue_options(
    env: &HostingEnv,
    target: &RepoTarget,
) -> Result<Emitted, String> {
    let options = env
        .client(&target.integration, &target.integrations)?
        .issue_create_options(&target.project)
        .await
        .map_err(|e| e.to_string())?;
    let line = |title: &str, choices: &[gitpoulpi_hosting::Choice], default: &Option<String>| {
        if choices.is_empty() {
            return String::new();
        }
        format!(
            "{title}: {}",
            choices
                .iter()
                .map(|c| format!(
                    "{}{}",
                    c.label,
                    if default.as_deref() == Some(c.id.as_str()) {
                        " (default)"
                    } else {
                        ""
                    }
                ))
                .collect::<Vec<_>>()
                .join(", ")
        )
    };
    let human = [
        line("types", &options.types, &options.default_type),
        line("lists", &options.lists, &options.default_list),
    ]
    .into_iter()
    .filter(|l| !l.is_empty())
    .collect::<Vec<_>>()
    .join("\n");
    let human = if human.is_empty() {
        "this project offers no issue types or lists".to_string()
    } else {
        human
    };
    let mut json = target_json(target);
    for (key, value) in [
        ("types", json!(options.types)),
        ("lists", json!(options.lists)),
        ("default_type", json!(options.default_type)),
        ("default_list", json!(options.default_list)),
    ] {
        json[key] = value;
    }
    Ok(Emitted {
        exit_code: 0,
        json,
        human,
    })
}

/// The web page of `reference`: a number (`12`, `#12`, `!12`) for a pull
/// request; for an issue also the provider's own spelling (`AB#12`,
/// `ABC-12`, a Trello short link).
pub(super) fn open_url(
    target: &RepoTarget,
    kind: OpenKind,
    reference: &str,
) -> Result<Emitted, String> {
    let reference = reference.trim();
    let label = target.integration.kind.label();
    let no_scheme = || format!("{label} has no web URL scheme for this");
    let numeric = |text: &str| -> Option<u64> {
        let text = text.trim_start_matches(['#', '!']);
        let text = text
            .get(..3)
            .filter(|p| p.eq_ignore_ascii_case("AB#"))
            .map(|_| &text[3..])
            .unwrap_or(text);
        text.parse().ok()
    };
    let (url, number) = match kind {
        OpenKind::Pr => {
            let number = numeric(reference).ok_or_else(|| {
                format!("`{reference}` is not a pull request number (like 12, #12 or !12)")
            })?;
            (
                pull_request_web_url(&target.integration, &target.project, number),
                Some(number),
            )
        }
        OpenKind::Issue => {
            let family = target.integration.kind.family();
            let own_spelling = matches!(
                family,
                ApiFamily::Jira | ApiFamily::Trello | ApiFamily::Linear | ApiFamily::YouTrack
            );
            let text = if own_spelling {
                // Put into the URL as typed, so only a key/short-link alphabet.
                if reference.is_empty()
                    || !reference
                        .chars()
                        .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
                {
                    return Err(format!(
                        "`{reference}` is not an issue reference of {label}"
                    ));
                }
                reference.to_string()
            } else {
                numeric(reference)
                    .ok_or_else(|| {
                        format!("`{reference}` is not an issue number (like 12, #12 or AB#12)")
                    })?
                    .to_string()
            };
            if matches!(
                family,
                ApiFamily::Jira | ApiFamily::Linear | ApiFamily::YouTrack
            ) && !is_jira_key(&text)
            {
                return Err(format!(
                    "`{reference}` is not an issue key of {label} (like ABC-12)"
                ));
            }
            let base = gitpoulpi_hosting::integration_web_base(&target.integration)
                .ok_or_else(no_scheme)?;
            (
                (spec_for(family).issue_web_url)(&base, &target.project, &text),
                text.parse().ok(),
            )
        }
    };
    let url = url.ok_or_else(no_scheme)?;
    let mut json = merge(target_json(target), "reference", json!(reference));
    json["url"] = json!(url);
    if let Some(number) = number {
        json["number"] = json!(number);
    }
    Ok(Emitted {
        exit_code: 0,
        json,
        human: url,
    })
}

/// `ABC-12` (Jira, Linear, YouTrack): an uppercase key, a dash, digits.
fn is_jira_key(text: &str) -> bool {
    text.split_once('-').is_some_and(|(key, n)| {
        key.chars().next().is_some_and(|c| c.is_ascii_uppercase())
            && key
                .chars()
                .all(|c| c.is_ascii_uppercase() || c.is_ascii_digit() || c == '_')
            && !n.is_empty()
            && n.chars().all(|c| c.is_ascii_digit())
    })
}

pub(super) fn open_in_browser(url: &str) -> Result<(), String> {
    gitpoulpi_config::browser::open_url(url).map_err(|e| e.to_string())
}

/// `(remote ref, local branch)` a pull request is fetched as. GitHub and
/// GitLab keep its head in a special ref of the target repository; Azure
/// DevOps and Bitbucket Cloud have only the source branch, so the request is
/// looked up first. A request from a fork is refused: its branch lives in
/// another repository, and fetching it would need a remote GitPoulpi will
/// not add behind the user's back.
pub(super) async fn checkout_ref(
    env: &HostingEnv,
    target: &RepoTarget,
    number: u64,
) -> Result<(String, String), String> {
    if let Some(found) = gitpoulpi_hosting::pull_request_head_ref(target.integration.kind, number) {
        return Ok(found);
    }
    let all = env
        .client(&target.integration, &target.integrations)?
        .list_pull_requests(
            &target.project,
            &PrFilter {
                state: StateFilter::All,
                ..Default::default()
            },
        )
        .await
        .map_err(|e| e.to_string())?;
    let pr = all
        .into_iter()
        .find(|p| p.number == number)
        .ok_or_else(|| {
            format!(
                "no pull request {number} on {}; nothing was fetched",
                target.project
            )
        })?;
    match (spec_for(target.integration.kind.family()).checkout)(&pr) {
        gitpoulpi_hosting::CheckoutSource::HeadRef { remote_ref, local } => Ok((remote_ref, local)),
        gitpoulpi_hosting::CheckoutSource::Branch {
            fork_url: Some(fork),
            ..
        } => Err(format!(
            "pull request {number} comes from the fork {fork}; add the fork as a remote and \
             check its branch `{}` out instead",
            pr.source_branch
        )),
        gitpoulpi_hosting::CheckoutSource::Branch {
            branch: source,
            local,
            ..
        } => Ok((format!("refs/heads/{source}"), local)),
    }
}
