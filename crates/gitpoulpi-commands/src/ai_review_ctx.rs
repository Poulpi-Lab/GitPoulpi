//! `gitpoulpi ai review-context`: the defaults of what a code review draft is
//! given besides the diff (`[ai.review]` and the per-repository values), and
//! the helpers `hosting review suggest` and `ai prompt` read them with.
//!
//! Three places hold the values: `config.toml`'s `[ai.review]` (global), the
//! repository's `repo-state.toml` (`preferences.ai_review`) and, for a
//! repository that is not cloned, `<config dir>/remote-reviews.toml` keyed by
//! `host/project`. A `None` field of the last two inherits the level above.

use std::path::{Path, PathBuf};

use gitpoulpi_ai::review_context::{resolve, EffectiveReviewContext};
use gitpoulpi_config::{AiReviewContext, ConfigPaths, RepoAiReview, Settings};
use gitpoulpi_session::{RemoteReviews, RepoState};
use serde_json::{json, Value};

use crate::cli::ReviewFlags;
use crate::commands::emitted;
use crate::repo_cmd::Emitted;

fn config_paths() -> Result<ConfigPaths, String> {
    ConfigPaths::discover().ok_or_else(|| "could not determine this OS's config directory".into())
}

fn remote_file() -> Result<PathBuf, String> {
    Ok(RemoteReviews::file_in(config_paths()?.config_dir()))
}

/// The global `[ai.review]` table.
pub(crate) fn global() -> Result<AiReviewContext, String> {
    let settings = Settings::resolve_ambient().map_err(|e| e.to_string())?;
    Ok(settings.config().ai.review.clone())
}

/// A cloned repository's own values.
pub(crate) fn repo_values(root: &Path) -> RepoAiReview {
    RepoState::load(&crate::commands::repo_state_path(root))
        .map(|s| s.preferences.ai_review)
        .unwrap_or_default()
}

/// A project's own values, for a repository that is not cloned.
pub(crate) fn project_values(key: &str) -> RepoAiReview {
    remote_file()
        .map(|file| RemoteReviews::load(&file))
        .ok()
        .and_then(|r| r.ai_review.get(key).cloned())
        .unwrap_or_default()
}

/// `HOST/PROJECT` as `remote-reviews.toml` keys it.
pub(crate) fn project_key(arg: &str) -> Result<String, String> {
    match arg.trim().trim_matches('/').split_once('/') {
        Some((host, project)) if !host.is_empty() && !project.is_empty() => {
            Ok(RemoteReviews::project_key(host, project))
        }
        _ => Err(format!(
            "--project expects HOST/PROJECT (`github.com/owner/name`), got `{arg}`"
        )),
    }
}

/// What a review gets: the global values, the repository's (or the
/// project's) over them, `per_run` over both.
pub(crate) fn effective(
    root: Option<&Path>,
    project: Option<&str>,
    per_run: &RepoAiReview,
) -> Result<EffectiveReviewContext, String> {
    let global = global()?;
    let own = match (root, project) {
        (Some(root), _) => Some(repo_values(root)),
        (None, Some(key)) => Some(project_values(key)),
        (None, None) => None,
    };
    Ok(resolve(&global, own.as_ref(), per_run))
}

fn report(
    scope: &str,
    target: Option<String>,
    own: Option<&RepoAiReview>,
    effective: &EffectiveReviewContext,
    changed: bool,
) -> Result<Emitted, String> {
    let global = global()?;
    let human = format!(
        "{}\n\nrun in the repository: {}\nCLAUDE.md: {}\nAGENTS.md: {}\nextra file: {}",
        match &target {
            Some(t) => format!("{scope} {t}"),
            None => scope.to_string(),
        },
        effective.run_in_repository.key(),
        if effective.include_claude_md {
            "on"
        } else {
            "off"
        },
        if effective.include_agents_md {
            "on"
        } else {
            "off"
        },
        if effective.extra_file.is_empty() {
            if own.is_some_and(|o| o.no_extra_file == Some(true)) {
                "none (the repository's No extra file switch)"
            } else {
                "none"
            }
        } else {
            &effective.extra_file
        },
    );
    let mut out = json!({
        "scope": scope,
        "target": target,
        "global": global,
        "effective": effective,
        "changed": changed,
    });
    if let Some(own) = own {
        out["own"] = serde_json::to_value(own).map_err(|e| e.to_string())?;
    }
    Ok(emitted(out, human))
}

/// `ai review-context`: shows the values, or sets the ones the flags name.
pub(crate) fn run(
    repo: Option<&str>,
    project: Option<&str>,
    flags: &ReviewFlags,
    reset: bool,
) -> Result<Emitted, String> {
    if repo.is_some() && project.is_some() {
        return Err("pass --repo or --project, not both".into());
    }
    let delta = flags.values();
    let sets = !delta.is_empty();
    if reset && repo.is_none() && project.is_none() {
        return Err(
            "--reset needs --repo or --project (the global values have no default to fall back on)"
                .into(),
        );
    }
    if repo.is_none() && project.is_none() && delta.no_extra_file.is_some() {
        return Err(
            "--no-extra-file needs --repo or --project (globally, clear it with --no-context-file)"
                .into(),
        );
    }
    if project.is_some() && delta.run_in_repository.is_some() {
        return Err(
            "--run-in-repo does not apply to --project: a repository that is not cloned has no checkout"
                .into(),
        );
    }
    if let Some(repo) = repo {
        let root = crate::client::discover_repo(Some(repo))?.root;
        let path = crate::commands::repo_state_path(&root);
        let mut changed = false;
        if sets || reset {
            let mut state = RepoState::load(&path).map_err(|e| e.to_string())?;
            if reset {
                state.preferences.ai_review = RepoAiReview::default();
            }
            overlay(&mut state.preferences.ai_review, &delta);
            state.save(&path).map_err(|e| e.to_string())?;
            changed = true;
        }
        let own = repo_values(&root);
        let eff = resolve(&global()?, Some(&own), &RepoAiReview::default());
        return report(
            "repository",
            Some(root.display().to_string()),
            Some(&own),
            &eff,
            changed,
        );
    }
    if let Some(project) = project {
        let key = project_key(project)?;
        let file = remote_file()?;
        let mut changed = false;
        if sets || reset {
            let mut all = RemoteReviews::load(&file);
            let mut own = all.ai_review.remove(&key).unwrap_or_default();
            if reset {
                own = RepoAiReview::default();
            }
            overlay(&mut own, &delta);
            if !own.is_empty() {
                all.ai_review.insert(key.clone(), own);
            }
            all.save(&file).map_err(|e| e.to_string())?;
            changed = true;
        }
        let own = project_values(&key);
        let eff = resolve(&global()?, Some(&own), &RepoAiReview::default());
        return report("project", Some(key), Some(&own), &eff, changed);
    }
    let mut changed = false;
    if sets {
        let file = config_paths()?.config_file();
        let set = |key: &str, value: String| {
            gitpoulpi_config::set_in_file(&file, &format!("ai.review.{key}"), &value)
                .map(|_| ())
                .map_err(|e| e.to_string())
        };
        if let Some(v) = delta.run_in_repository {
            set("run_in_repository", v.key().to_string())?;
        }
        if let Some(v) = delta.include_claude_md {
            set("include_claude_md", v.to_string())?;
        }
        if let Some(v) = delta.include_agents_md {
            set("include_agents_md", v.to_string())?;
        }
        if let Some(v) = delta.extra_file.clone() {
            set("extra_file", v)?;
        }

        changed = true;
    }
    let eff = resolve(&global()?, None, &RepoAiReview::default());
    report("global", None, None, &eff, changed)
}

/// `delta`'s choices over `own`.
fn overlay(own: &mut RepoAiReview, delta: &RepoAiReview) {
    if delta.run_in_repository.is_some() {
        own.run_in_repository = delta.run_in_repository;
    }
    if delta.include_claude_md.is_some() {
        own.include_claude_md = delta.include_claude_md;
    }
    if delta.include_agents_md.is_some() {
        own.include_agents_md = delta.include_agents_md;
    }
    if delta.extra_file.is_some() {
        own.extra_file = delta.extra_file.clone();
        // A file chosen now replaces an earlier "no extra file".
        if delta.extra_file.as_deref().is_some_and(|f| !f.is_empty()) {
            own.no_extra_file = None;
        }
    }
    match delta.no_extra_file {
        Some(true) => {
            own.no_extra_file = Some(true);
            own.extra_file = None;
        }
        Some(false) => own.no_extra_file = None,
        None => {}
    }
}

/// The JSON of an effective context for `hosting review suggest` and
/// `ai prompt`.
pub(crate) fn effective_json(effective: &EffectiveReviewContext) -> Value {
    serde_json::to_value(effective).unwrap_or(Value::Null)
}
