use gitpoulpi_core::domain::{
    decode_commit_text, visible_tips, CommitGraph, CommitNode, LogOptions, Oid, RefFilter, RepoRef,
};
use gitpoulpi_core::Result;

use crate::process::{read_with_retry, run, run_bytes, run_bytes_with_stdin};
use crate::refs::peel_commits;

const FIELD_SEP: char = '\u{1}';
const RECORD_SEP: char = '\u{2}';

/// The commits a walk with a non-empty filter starts from: every branch,
/// tag and remote-tracking ref (annotated tags peeled; a tag of a tree or a
/// blob is no tip) the filter keeps, plus `HEAD` when it resolves. Empty
/// for an unborn repository with no ref at all. A listing that fails is an
/// error, never an empty list: the Log would silently walk only `HEAD`.
pub(crate) fn filtered_tips(repo: &RepoRef, filter: &RefFilter) -> Result<Vec<Oid>> {
    // Only `%(objectname)`: git 2.5x aborts (or exits 128) on a ref whose
    // object is missing when asked for its type or its peeled object.
    // `peel_commits` answers both from `cat-file`; a ref that does not reach
    // a commit (missing object, tag of a tree) is no tip.
    let listing = read_with_retry(|| {
        run(
            &repo.root,
            &[
                "for-each-ref",
                "--format=%(refname)%09%(objectname)%09%(symref)",
                "refs/heads",
                "refs/tags",
                "refs/remotes",
            ],
        )
    })?;
    let rows: Vec<(&str, Oid)> = listing
        .lines()
        .filter_map(|line| {
            let mut f = line.split('\t');
            let (name, id) = (f.next()?, f.next()?);
            // `refs/remotes/<r>/HEAD` and friends mirror another ref: not a
            // tip of their own (hiding the branch they name must hide it).
            if !f.next().unwrap_or_default().is_empty() {
                return None;
            }
            Some((name, Oid::from_hex(id.trim())?))
        })
        .collect();
    let mut ids: Vec<Oid> = rows.iter().map(|(_, id)| *id).collect();
    ids.sort_by_key(|id| id.0);
    ids.dedup();
    let peeled = peel_commits(&repo.root, &ids)?;
    let refs: Vec<(String, Oid)> = rows
        .into_iter()
        .filter_map(|(name, id)| Some((name.to_string(), *peeled.get(&id)?)))
        .collect();
    let head = run(
        &repo.root,
        &["rev-parse", "--verify", "-q", "HEAD^{commit}"],
    )
    .ok()
    .and_then(|out| Oid::from_hex(out.trim()));
    Ok(visible_tips(filter, &refs, head))
}

/// The tips as `--stdin` input: one id per line. A command line of
/// thousands of refs would break Windows' limit, and `--exclude` resets
/// after each `--branches`/`--tags` and takes per-family patterns.
pub(crate) fn tips_stdin(tips: &[Oid]) -> Vec<u8> {
    let mut text = String::new();
    for tip in tips {
        text.push_str(&tip.to_hex());
        text.push('\n');
    }
    text.into_bytes()
}

/// A detached HEAD is always walked, like the current branch: `HEAD` is
/// added to the revisions of an unfiltered all-refs walk so commits only a
/// detached HEAD reaches are not lost (on a branch it duplicates a ref the
/// walk has, which git dedupes; no extra spawn to ask). An unborn HEAD
/// makes git refuse it: `without_head` is the same arguments without it.
pub(crate) fn with_head(args: &[String]) -> Vec<String> {
    let mut with = args.to_vec();
    // Before the path separator when there is one; else add the separator,
    // so a file named `HEAD` cannot make the revision ambiguous.
    let at = with.iter().position(|a| a == "--").unwrap_or_else(|| {
        with.push("--".to_string());
        with.len() - 1
    });
    with.insert(at, "HEAD".to_string());
    with
}

/// Runs `args` with `HEAD` as an extra revision, else (unborn HEAD, other
/// branches present) without it.
pub(crate) fn run_with_head_fallback(
    args: &[String],
    mut go: impl FnMut(&[String]) -> Result<Vec<u8>>,
) -> Result<Vec<u8>> {
    go(&with_head(args)).or_else(|_| go(args))
}

pub fn log(repo: &RepoRef, opts: &LogOptions) -> Result<CommitGraph> {
    // `%aN`/`%aE` are `%an`/`%ae` through the `.mailmap` (`log.use_mailmap`).
    let (name, email) = if opts.use_mailmap {
        ("%aN", "%aE")
    } else {
        ("%an", "%ae")
    };
    let format = format!(
        "%H{FIELD_SEP}%P{FIELD_SEP}%s{FIELD_SEP}{name}{FIELD_SEP}{email}{FIELD_SEP}%at{FIELD_SEP}%B{RECORD_SEP}"
    );

    // `--topo-order`: `git log` alone only guarantees reverse-chronological
    // order, so on diverged branches a parent could surface before its
    // child and break the TUI's lane bookkeeping (parents must come later
    // in `nodes`). Matches the git2 backend's `TIME | TOPOLOGICAL` sort.
    let mut args = vec![
        "log".to_string(),
        "--topo-order".to_string(),
        format!("--format={format}"),
    ];
    // `--skip` before `-n`: git applies skip first and then counts `-n`
    // regardless of command-line order, which is the paging semantics
    // `check_log_skip_pages_history` pins and the git2 revwalk mirrors.
    if opts.skip > 0 {
        args.push(format!("--skip={}", opts.skip));
    }
    if let Some(limit) = opts.limit {
        args.push(format!("-n{limit}"));
    }
    let mut stdin: Option<Vec<u8>> = None;
    let mut head_seed = false;
    if opts.all_refs && !opts.hidden.is_empty() {
        // Hidden refs change the walk: start from the visible tips only.
        // `--stdin` with no input would fall back to HEAD, so an empty list
        // is an empty graph without running git.
        let tips = filtered_tips(repo, &opts.hidden)?;
        if tips.is_empty() {
            return Ok(CommitGraph::default());
        }
        args.push("--stdin".to_string());
        stdin = Some(tips_stdin(&tips));
    } else if opts.all_refs {
        // `--branches --tags --remotes`, not `--all`: `--all` would include
        // GitPoulpi's remote-tag mirror (`refs/gitpoulpi/remote-tags/...`),
        // which must never put a line in the commit graph. These match
        // `Git2Backend`'s three `push_glob`s.
        args.push("--branches".to_string());
        args.push("--tags".to_string());
        args.push("--remotes".to_string());
        head_seed = true;
    } else if let Some(start) = &opts.start_ref {
        args.push(start.clone());
    }
    if !opts.paths.is_empty() {
        // `--` first: without the separator a path that is also a ref name
        // makes git pick the ref or refuse as "ambiguous". Backslashes are
        // normalized because git speaks `/` on every platform.
        args.push("--".to_string());
        args.extend(
            opts.paths
                .iter()
                .map(|p| p.to_string_lossy().replace('\\', "/")),
        );
    }
    let go = |args: &[String]| {
        let args_ref: Vec<&str> = args.iter().map(String::as_str).collect();
        match &stdin {
            Some(input) => run_bytes_with_stdin(&repo.root, &args_ref, input),
            None => run_bytes(&repo.root, &args_ref),
        }
    };
    let result = if head_seed {
        run_with_head_fallback(&args, go)
    } else {
        go(&args)
    };
    let raw = match result {
        Ok(raw) => raw,
        // an empty repo (no commits yet) makes `git log` fail -- that's a
        // valid state, not an error.
        Err(_) => return Ok(CommitGraph::default()),
    };

    // Bytes, not a `String`: a commit message need not be UTF-8. git has
    // already converted commits that declare an `encoding` header to UTF-8;
    // what is left that is not UTF-8 is read with the fallback encoding
    // (`git.default_encoding`), field by field -- both separators are ASCII
    // control bytes no multi-byte encoding produces inside a character.
    let text = |bytes: &[u8]| decode_commit_text(bytes, None).into_owned();
    let nodes = raw
        .split(|b| *b == RECORD_SEP as u8)
        .map(<[u8]>::trim_ascii)
        .filter(|record| !record.is_empty())
        .filter_map(|record| {
            // splitn, not split: %B (the last field) is the full commit
            // message and may itself contain blank lines/paragraphs, but
            // must never be split further -- it's whatever's left after
            // the six fixed-width fields ahead of it.
            let mut fields = record.splitn(7, |b| *b == FIELD_SEP as u8);
            let id = Oid::from_hex(text(fields.next()?).trim())?;
            let parents = text(fields.next()?)
                .split_whitespace()
                .filter_map(Oid::from_hex)
                .collect();
            let summary = text(fields.next()?);
            let author_name = text(fields.next()?);
            let author_email = text(fields.next()?);
            let timestamp = text(fields.next()?).trim().parse().ok()?;
            let message = text(fields.next()?).trim_end().to_string();

            Some(CommitNode {
                id,
                parents,
                summary,
                message,
                author_name,
                author_email,
                timestamp,
                refs: Vec::new(),
            })
        })
        .collect();

    Ok(CommitGraph { nodes })
}

#[cfg(test)]
mod filtered_tips_tests {
    use super::*;

    /// A listing that fails is an error: an empty list would make the Log
    /// walk only `HEAD` while claiming to show the filtered graph.
    #[test]
    fn a_failing_ref_listing_is_an_error_not_an_empty_list() {
        let not_a_repo = tempfile::tempdir().unwrap();
        let repo = RepoRef::from_root(not_a_repo.path());
        let filter = RefFilter {
            names: vec!["main".into()],
            ..RefFilter::default()
        };
        let result = filtered_tips(&repo, &filter);
        assert!(result.is_err(), "got {result:?}");
    }
}
