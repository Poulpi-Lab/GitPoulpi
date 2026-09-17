//! `commit_messages` over `git for-each-ref` and `git log --stdin`.
//!
//! The tips go through stdin (`tip` / `^known` lines): a repository with
//! thousands of tags would overflow an argument list. Same set of refs and
//! the same decisions as the git2 backend (`reads/messages.rs`).

use gitpoulpi_core::domain::{
    decode_commit_text, split_message, CommitMessage, CommitMessages, CommitMessagesQuery, Oid,
    RepoRef,
};
use gitpoulpi_core::Result;

use crate::process::{run, run_bytes_with_stdin, run_lenient};

const FIELD_SEP: u8 = 1;
const RECORD_SEP: u8 = 2;

/// Branches, remote-tracking branches, tags (peeled, commits only) and HEAD.
fn tips(repo: &RepoRef) -> Result<Vec<Oid>> {
    let listing = run(
        &repo.root,
        &[
            "for-each-ref",
            "--format=%(objectname) %(objecttype) %(*objectname) %(*objecttype)",
            "refs/heads",
            "refs/remotes",
            "refs/tags",
        ],
    )?;
    let mut out: Vec<Oid> = Vec::new();
    // Tags of tags that `%(*objectname)` peeled only one level (older git
    // stops at the inner tag): resolved to their commit below.
    let mut nested: Vec<&str> = Vec::new();
    for line in listing.lines() {
        let mut f = line.split(' ');
        let (Some(name), Some(kind), Some(peeled), Some(peeled_kind)) =
            (f.next(), f.next(), f.next(), f.next())
        else {
            continue;
        };
        let oid = match (kind, peeled_kind) {
            ("commit", _) => Oid::from_hex(name),
            // `*` fields peel a tag of a tag all the way, as libgit2 does.
            ("tag", "commit") => Oid::from_hex(peeled),
            ("tag", "tag") => {
                nested.push(name);
                None
            }
            _ => None,
        };
        out.extend(oid);
    }
    if !nested.is_empty() {
        let mut stdin = String::new();
        for tag in nested {
            stdin.push_str(tag);
            stdin.push_str("^{commit}\n");
        }
        let resolved = run_bytes_with_stdin(
            &repo.root,
            &["cat-file", "--batch-check=%(objectname) %(objecttype)"],
            stdin.as_bytes(),
        )?;
        for line in String::from_utf8_lossy(&resolved).lines() {
            if let Some(id) = line.strip_suffix(" commit") {
                out.extend(Oid::from_hex(id));
            }
        }
    }
    let (ok, head) = run_lenient(
        &repo.root,
        &["rev-parse", "-q", "--verify", "HEAD^{commit}"],
    )?;
    if ok {
        out.extend(Oid::from_hex(head.trim()));
    }
    out.sort_by_key(|o| o.0);
    out.dedup();
    Ok(out)
}

fn stdin_of(include: &[Oid], exclude: &[Oid]) -> String {
    let mut s = String::with_capacity((include.len() + exclude.len()) * 42);
    for t in include {
        s.push_str(&t.to_hex());
        s.push('\n');
    }
    for k in exclude {
        s.push('^');
        s.push_str(&k.to_hex());
        s.push('\n');
    }
    s
}

pub fn commit_messages(repo: &RepoRef, query: &CommitMessagesQuery) -> Result<CommitMessages> {
    let tips = tips(repo)?;
    let mut out = CommitMessages {
        tips: tips.clone(),
        ..CommitMessages::default()
    };
    if tips.is_empty() {
        return Ok(out);
    }

    // Is anything the known tips reach no longer reachable from the current
    // ones? `rev-list` prints a commit if so; a known tip the object
    // database lost makes it fail, which is also a rewrite.
    out.replace = !query.known_tips.is_empty()
        && match run_bytes_with_stdin(
            &repo.root,
            &["rev-list", "-n1", "--stdin"],
            // Known tips positive, current tips negated.
            stdin_of(&query.known_tips, &tips).as_bytes(),
        ) {
            Ok(printed) => !printed.trim_ascii().is_empty(),
            Err(_) => true,
        };
    let exclude: &[Oid] = if out.replace { &[] } else { &query.known_tips };

    // One more than asked, to know whether the limit cut the walk.
    let format = "--format=%H\u{1}%an\u{1}%ct\u{1}%B\u{2}";
    let limit_arg = (query.limit > 0).then(|| format!("-n{}", query.limit + 1));
    let mut args = vec!["log", format, "--stdin"];
    if let Some(limit) = &limit_arg {
        args.push(limit);
    }
    let raw = run_bytes_with_stdin(&repo.root, &args, stdin_of(&tips, exclude).as_bytes())?;
    for record in raw
        .split(|b| *b == RECORD_SEP)
        .map(<[u8]>::trim_ascii)
        .filter(|r| !r.is_empty())
    {
        if query.limit > 0 && out.commits.len() >= query.limit {
            out.truncated = true;
            break;
        }
        let mut fields = record.splitn(4, |b| *b == FIELD_SEP);
        let (Some(id), Some(author), Some(time), Some(message)) =
            (fields.next(), fields.next(), fields.next(), fields.next())
        else {
            continue;
        };
        let Some(id) = std::str::from_utf8(id).ok().and_then(Oid::from_hex) else {
            continue;
        };
        let message = decode_commit_text(message, None);
        let (subject, body) = split_message(&message);
        out.commits.push(CommitMessage {
            id,
            author: decode_commit_text(author, None).into_owned(),
            time: std::str::from_utf8(time)
                .ok()
                .and_then(|t| t.trim().parse().ok())
                .unwrap_or(0),
            subject,
            body,
        });
    }
    Ok(out)
}
