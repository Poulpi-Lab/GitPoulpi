use gitpoulpi_core::domain::{
    decode_commit_text, CommitGraph, CommitNode, LogOptions, Oid, RepoRef,
};
use gitpoulpi_core::Result;

use crate::process::run_bytes;

const FIELD_SEP: char = '\u{1}';
const RECORD_SEP: char = '\u{2}';

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
    if opts.all_refs {
        // `--branches --tags --remotes`, not `--all`: `--all` would include
        // GitPoulpi's remote-tag mirror (`refs/gitpoulpi/remote-tags/...`),
        // which must never put a line in the commit graph. These match
        // `Git2Backend`'s three `push_glob`s.
        args.push("--branches".to_string());
        args.push("--tags".to_string());
        args.push("--remotes".to_string());
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
    let args_ref: Vec<&str> = args.iter().map(String::as_str).collect();

    let raw = match run_bytes(&repo.root, &args_ref) {
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
