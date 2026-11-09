//! The project context a code review draft is given next to the diff:
//! the repository's `CLAUDE.md` and `AGENTS.md`, and one more file (a path
//! or a link), read at the branch the request targets.
//!
//! Three levels decide what is included, most specific last: the global
//! `[ai.review]` table, the repository's own values
//! (`RepoPreferences::ai_review`, or `remote-reviews.toml` for a repository
//! that is not cloned) and the choices of one run ([`resolve`]).
//!
//! Reading is behind [`ContextSource`], so the TUI (the daemon's `ReadBlob`
//! of the base ref, or the host's `file_content`) and the CLI share the
//! rules and the caps without sharing a transport.

use std::path::{Component, Path};

use async_trait::async_trait;
use gitpoulpi_config::{
    AiReviewContext, ApiFamily, Integration, IntegrationCategory, RepoAiReview, RunInRepo,
};
use gitpoulpi_hosting::{HttpRequest, HttpTransport};

use crate::limits::{Cut, Part};

/// How much of one context file reaches the model, in bytes, by default
/// (`ai.limits.review_context_file_bytes`).
pub const MAX_FILE_BYTES: usize = gitpoulpi_config::DEFAULT_REVIEW_CONTEXT_FILE_BYTES;

/// How much all the context files reach the model together, in bytes, by
/// default (`ai.limits.review_context_bytes`).
pub const MAX_TOTAL_BYTES: usize = gitpoulpi_config::DEFAULT_REVIEW_CONTEXT_BYTES;

/// What a review is given, after the three levels are applied.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct EffectiveReviewContext {
    pub run_in_repository: RunInRepo,
    pub include_claude_md: bool,
    pub include_agents_md: bool,
    /// A path (relative to the repository root, or absolute) or a link;
    /// empty: none.
    pub extra_file: String,
}

/// The global values, then the repository's over them, then the run's over
/// those. A `None` inherits the level below.
pub fn resolve(
    global: &AiReviewContext,
    repo_override: Option<&RepoAiReview>,
    per_run: &RepoAiReview,
) -> EffectiveReviewContext {
    let empty = RepoAiReview::default();
    let repo = repo_override.unwrap_or(&empty);
    EffectiveReviewContext {
        run_in_repository: per_run
            .run_in_repository
            .or(repo.run_in_repository)
            .unwrap_or(global.run_in_repository),
        include_claude_md: per_run
            .include_claude_md
            .or(repo.include_claude_md)
            .unwrap_or(global.include_claude_md),
        include_agents_md: per_run
            .include_agents_md
            .or(repo.include_agents_md)
            .unwrap_or(global.include_agents_md),
        extra_file: per_run
            .extra_choice()
            .or_else(|| repo.extra_choice())
            .unwrap_or_else(|| global.extra_file.clone())
            .trim()
            .to_string(),
    }
}

impl EffectiveReviewContext {
    /// What a review is given, in words: the "context layer" `ai prompt`
    /// prints for `code_review`.
    pub fn describe(&self) -> String {
        let mut files = Vec::new();
        if self.include_claude_md {
            files.push("CLAUDE.md".to_string());
        }
        if self.include_agents_md {
            files.push("AGENTS.md".to_string());
        }
        if !self.extra_file.is_empty() {
            files.push(self.extra_file.clone());
        }
        let given = if files.is_empty() {
            "no project conventions are added".to_string()
        } else {
            format!(
                "a \"Project conventions\" block with {} is added before the diff \
                 (root files read at the branch the request targets; {} KB per file, {} KB in total)",
                files.join(", "),
                MAX_FILE_BYTES / 1024,
                MAX_TOTAL_BYTES / 1024
            )
        };
        let runs = match self.run_in_repository {
            RunInRepo::Profile => "a local agent runs where its AI profile says",
            RunInRepo::Yes => "a local agent runs in the repository",
            RunInRepo::No => "a local agent does not run in the repository",
        };
        format!("{given}; {runs}.")
    }
}

/// One file given to the model.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContextFile {
    /// `CLAUDE.md`, `AGENTS.md`, or the extra file as the user wrote it.
    pub label: String,
    /// The text, already shortened to the caps; when [`Self::cut`] it ends
    /// with a visible note saying so.
    pub text: String,
    /// The file did not fit the caps.
    pub cut: bool,
}

/// What [`gather_report`] found.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Gathered {
    pub files: Vec<ContextFile>,
    /// Files that were asked for and could not be given (an extra file that
    /// does not exist or cannot be fetched, a file that no longer fits the
    /// total cap). `CLAUDE.md` and `AGENTS.md` missing is not a problem.
    pub problems: Vec<String>,
    /// Every file that was shortened or left out because of a cap: what a
    /// caller asks about before sending, or reports.
    pub cuts: Vec<Cut>,
}

/// Where the files come from. Every read answers `Ok(None)` for a file that
/// does not exist, and `Err` for one that could not be read.
#[async_trait]
pub trait ContextSource: Send + Sync {
    /// A file of the repository's root at the branch the request targets
    /// (the daemon's `ReadBlob` of the base ref; the host's `file_content`
    /// at the base ref when there is no clone).
    async fn repo_file(&self, path: &str) -> Result<Option<Vec<u8>>, String>;

    /// A file on this machine, by absolute path.
    async fn disk_file(&self, path: &Path) -> Result<Option<Vec<u8>>, String> {
        match std::fs::read(path) {
            Ok(bytes) => Ok(Some(bytes)),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
            Err(e) => Err(e.to_string()),
        }
    }

    /// A file by link (see [`fetch_link`]).
    async fn link_file(&self, url: &str) -> Result<Option<Vec<u8>>, String>;
}

/// How an extra file is named.
#[derive(Debug, Clone, PartialEq, Eq)]
enum ExtraKind<'a> {
    Link(&'a str),
    Absolute(&'a Path),
    /// A path in the repository; already checked to stay inside it.
    Relative(String),
    /// A relative path that leaves the repository (`..`).
    Escapes,
}

fn classify(extra: &str) -> ExtraKind<'_> {
    let lower = extra.to_ascii_lowercase();
    if lower.starts_with("https://") || lower.starts_with("http://") {
        return ExtraKind::Link(extra);
    }
    let path = Path::new(extra);
    if path.is_absolute() {
        return ExtraKind::Absolute(path);
    }
    let mut parts: Vec<String> = Vec::new();
    for component in path.components() {
        match component {
            Component::Normal(part) => parts.push(part.to_string_lossy().into_owned()),
            Component::CurDir => {}
            _ => return ExtraKind::Escapes,
        }
    }
    if parts.is_empty() {
        ExtraKind::Escapes
    } else {
        ExtraKind::Relative(parts.join("/"))
    }
}

/// `bytes` as text shortened to `limit` bytes (on a character boundary),
/// with a visible note when it was cut.
fn capped(label: &str, bytes: &[u8], limit: usize) -> (String, bool) {
    let text = String::from_utf8_lossy(bytes);
    let text = text.trim_end();
    if text.len() <= limit {
        return (text.to_string(), false);
    }
    let mut end = limit;
    while end > 0 && !text.is_char_boundary(end) {
        end -= 1;
    }
    (
        format!(
            "{}\n[{label} cut: showing {end} of {} bytes]",
            &text[..end],
            text.len()
        ),
        true,
    )
}

/// Reads the files `ctx` asks for, in the order `CLAUDE.md`, `AGENTS.md`,
/// the extra file, within [`MAX_FILE_BYTES`] each and [`MAX_TOTAL_BYTES`]
/// together.
pub async fn gather(ctx: &EffectiveReviewContext, source: &dyn ContextSource) -> Vec<ContextFile> {
    gather_report(ctx, source).await.files
}

/// [`gather`], plus what could not be given.
pub async fn gather_report(ctx: &EffectiveReviewContext, source: &dyn ContextSource) -> Gathered {
    gather_within(ctx, source, MAX_FILE_BYTES, MAX_TOTAL_BYTES).await
}

/// [`gather_report`] with the caps of `[ai.limits]` (`usize::MAX`: none).
pub async fn gather_within(
    ctx: &EffectiveReviewContext,
    source: &dyn ContextSource,
    file_limit: usize,
    total_limit: usize,
) -> Gathered {
    let mut out = Gathered::default();
    let mut used = 0usize;
    let mut take = |label: String, found: Result<Option<Vec<u8>>, String>, required: bool| {
        let bytes = match found {
            Ok(Some(bytes)) => bytes,
            Ok(None) => {
                if required {
                    out.problems.push(format!("{label}: not found"));
                }
                return;
            }
            Err(e) => {
                if required {
                    out.problems.push(format!("{label}: {e}"));
                }
                return;
            }
        };
        if String::from_utf8_lossy(&bytes).trim().is_empty() {
            return;
        }
        let size = String::from_utf8_lossy(&bytes).trim_end().len();
        let remaining = total_limit.saturating_sub(used);
        if remaining == 0 {
            out.problems.push(format!(
                "{label}: left out, the context is over {total_limit} bytes"
            ));
            out.cuts.push(Cut::new(
                Part::ContextTotal,
                label,
                used.saturating_add(size),
                total_limit,
            ));
            return;
        }
        let allowed = file_limit.min(remaining);
        let (text, cut) = capped(&label, &bytes, allowed);
        if cut {
            // The total ran out before the file's own cap did: that cap is
            // the one to raise.
            out.cuts.push(if remaining < file_limit {
                Cut::new(
                    Part::ContextTotal,
                    label.clone(),
                    used.saturating_add(size),
                    total_limit,
                )
            } else {
                Cut::new(Part::ContextFile, label.clone(), size, file_limit)
            });
        }
        used += bytes.len().min(allowed);
        out.files.push(ContextFile { label, text, cut });
    };
    if ctx.include_claude_md {
        take(
            "CLAUDE.md".into(),
            source.repo_file("CLAUDE.md").await,
            false,
        );
    }
    if ctx.include_agents_md {
        take(
            "AGENTS.md".into(),
            source.repo_file("AGENTS.md").await,
            false,
        );
    }
    let extra = ctx.extra_file.trim();
    if !extra.is_empty() {
        let found = match classify(extra) {
            ExtraKind::Link(url) => source.link_file(url).await,
            ExtraKind::Absolute(path) => source.disk_file(path).await,
            ExtraKind::Relative(path) => source.repo_file(&path).await,
            ExtraKind::Escapes => Err("a relative path must stay inside the repository".into()),
        };
        take(extra.to_string(), found, true);
    }
    out
}

/// The host family a file link belongs to.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LinkKind {
    Github,
    Gitlab,
    /// Gitea and Forgejo (incl. Codeberg).
    Gitea,
}

impl LinkKind {
    fn family(self) -> ApiFamily {
        match self {
            Self::Github => ApiFamily::Github,
            Self::Gitlab => ApiFamily::Gitlab,
            Self::Gitea => ApiFamily::Gitea,
        }
    }
}

/// A link to one file of a project: `https://host/owner/repo/blob/REV/path`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FileLink {
    pub kind: LinkKind,
    /// `http` or `https`.
    pub scheme: String,
    /// `host[:port]`, lowercase. A GitHub raw link (`raw.githubusercontent.com`)
    /// names `github.com`.
    pub host: String,
    /// `owner/name`, `group/sub/project`.
    pub project: String,
    /// A branch, tag or commit (one path segment: a branch name with a slash
    /// in a link is read up to the slash).
    pub rev: String,
    pub path: String,
    /// Gitea only: `branch`, `tag` or `commit`.
    ref_kind: &'static str,
}

impl FileLink {
    /// The address of the file's raw bytes, for an anonymous read.
    pub fn raw_url(&self) -> String {
        let (scheme, host) = (&self.scheme, &self.host);
        let (project, rev, path) = (&self.project, &self.rev, &self.path);
        match self.kind {
            LinkKind::Github if host == "github.com" => {
                format!("https://raw.githubusercontent.com/{project}/{rev}/{path}")
            }
            LinkKind::Github => format!("{scheme}://{host}/{project}/raw/{rev}/{path}"),
            LinkKind::Gitlab => format!("{scheme}://{host}/{project}/-/raw/{rev}/{path}"),
            LinkKind::Gitea => format!(
                "{scheme}://{host}/{project}/raw/{}/{rev}/{path}",
                self.ref_kind
            ),
        }
    }
}

fn percent_decode(text: &str) -> String {
    let bytes = text.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'%' && i + 2 < bytes.len() {
            let hex = std::str::from_utf8(&bytes[i + 1..i + 3]).ok();
            if let Some(v) = hex.and_then(|h| u8::from_str_radix(h, 16).ok()) {
                out.push(v);
                i += 3;
                continue;
            }
        }
        out.push(bytes[i]);
        i += 1;
    }
    String::from_utf8_lossy(&out).into_owned()
}

/// Reads a file link: GitHub `blob/` and `raw/`, `raw.githubusercontent.com`,
/// GitLab `-/blob/` and `-/raw/`, Gitea/Forgejo `src/branch|tag|commit/` and
/// `raw/branch|tag|commit/`. Anything else is `None`.
pub fn parse_file_link(url: &str) -> Option<FileLink> {
    let url = url.trim();
    let (scheme, rest) = url.split_once("://")?;
    let scheme = scheme.to_ascii_lowercase();
    if scheme != "http" && scheme != "https" {
        return None;
    }
    let rest = rest.split(['?', '#']).next().unwrap_or(rest);
    let (authority, path) = rest.split_once('/')?;
    let authority = authority.rsplit('@').next().unwrap_or(authority);
    let host = authority.to_ascii_lowercase();
    if host.is_empty() {
        return None;
    }
    let segs: Vec<String> = path
        .split('/')
        .filter(|s| !s.is_empty())
        .map(percent_decode)
        .collect();
    let make = |kind, host: String, project: &[String], rev: &str, file: &[String], ref_kind| {
        if project.is_empty() || rev.is_empty() || file.is_empty() {
            return None;
        }
        Some(FileLink {
            kind,
            scheme: scheme.clone(),
            host,
            project: project.join("/"),
            rev: rev.to_string(),
            path: file.join("/"),
            ref_kind,
        })
    };
    // GitLab: `<project>/-/blob/<rev>/<path>`.
    if let Some(i) = (1..segs.len().saturating_sub(3))
        .find(|&i| segs[i] == "-" && (segs[i + 1] == "blob" || segs[i + 1] == "raw"))
    {
        return make(
            LinkKind::Gitlab,
            host,
            &segs[..i],
            &segs[i + 2],
            &segs[i + 3..],
            "",
        );
    }
    // Gitea: `<project>/src|raw/branch|tag|commit/<rev>/<path>`.
    if let Some(i) = (1..segs.len().saturating_sub(3)).find(|&i| {
        (segs[i] == "src" || segs[i] == "raw")
            && matches!(segs[i + 1].as_str(), "branch" | "tag" | "commit")
    }) {
        let ref_kind = match segs[i + 1].as_str() {
            "tag" => "tag",
            "commit" => "commit",
            _ => "branch",
        };
        return make(
            LinkKind::Gitea,
            host,
            &segs[..i],
            &segs[i + 2],
            &segs[i + 3..],
            ref_kind,
        );
    }
    // GitHub raw host: `<owner>/<repo>/<rev>/<path>`.
    if host == "raw.githubusercontent.com" && segs.len() >= 4 {
        let (rev, file) = if segs[2] == "refs" && segs.len() >= 6 && segs[3] == "heads" {
            (&segs[4], &segs[5..])
        } else {
            (&segs[2], &segs[3..])
        };
        return make(
            LinkKind::Github,
            "github.com".into(),
            &segs[..2],
            rev,
            file,
            "",
        );
    }
    // GitHub (and Enterprise): `<owner>/<repo>/blob|raw/<rev>/<path>`.
    if segs.len() >= 5 && (segs[2] == "blob" || segs[2] == "raw") {
        return make(LinkKind::Github, host, &segs[..2], &segs[3], &segs[4..], "");
    }
    None
}

/// Lowercase, with GitHub's `www.` and `api.` names folded into `github.com`.
fn canonical_host(host: &str) -> String {
    let host = host.to_ascii_lowercase();
    match host.as_str() {
        "www.github.com" | "api.github.com" => "github.com".to_string(),
        _ => host,
    }
}

/// Where a link is read.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LinkRoute<'a> {
    /// Through the integration whose host the link names (its token applies).
    Integration {
        integration: &'a Integration,
        link: FileLink,
    },
    /// An anonymous GET of this address.
    Anonymous { url: String },
}

/// Picks how `url` is read: through the enabled code-host profile whose host
/// and family match the link, else an anonymous GET (of the file's raw
/// address when the link is a recognised page of a host, of `url` itself
/// otherwise).
pub fn route_link<'a>(url: &str, integrations: &'a [Integration]) -> LinkRoute<'a> {
    let url = url.trim();
    let Some(link) = parse_file_link(url) else {
        return LinkRoute::Anonymous {
            url: url.to_string(),
        };
    };
    let matching = integrations.iter().find(|i| {
        i.enabled
            && i.kind.category() == IntegrationCategory::Hosting
            && i.kind.family() == link.kind.family()
            && i.host()
                .is_some_and(|h| canonical_host(&h) == canonical_host(&link.host))
    });
    match matching {
        Some(integration) => LinkRoute::Integration { integration, link },
        None => LinkRoute::Anonymous {
            url: link.raw_url(),
        },
    }
}

/// What reads a file through a code-host profile (its token applies).
#[async_trait]
pub trait ProfileReader: Send + Sync {
    /// The bytes of `path` of `project` at `rev`; `Ok(None)` when the file
    /// does not exist there.
    async fn read(
        &self,
        integration: &Integration,
        project: &str,
        path: &str,
        rev: &str,
    ) -> Result<Option<Vec<u8>>, String>;
}

/// An anonymous GET; `Ok(None)` on 404.
pub async fn fetch_anonymous(
    transport: &dyn HttpTransport,
    url: &str,
) -> Result<Option<Vec<u8>>, String> {
    let response = transport
        .send(HttpRequest::get(url))
        .await
        .map_err(|e| e.to_string())?;
    if response.status == 404 {
        return Ok(None);
    }
    response
        .error_for_status("reading a context file")
        .map_err(|e| e.to_string())?;
    Ok(Some(response.body.into_bytes()))
}

/// Reads `url`: through the profile matching its host ([`route_link`]), else
/// anonymously through `transport`.
pub async fn fetch_link(
    url: &str,
    integrations: &[Integration],
    profiles: &dyn ProfileReader,
    transport: &dyn HttpTransport,
) -> Result<Option<Vec<u8>>, String> {
    match route_link(url, integrations) {
        LinkRoute::Integration { integration, link } => {
            profiles
                .read(integration, &link.project, &link.path, &link.rev)
                .await
        }
        LinkRoute::Anonymous { url } => fetch_anonymous(transport, &url).await,
    }
}

#[cfg(test)]
mod tests;
