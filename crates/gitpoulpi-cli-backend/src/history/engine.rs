//! The rewrite: the scope picks the refs and the commits, the rule stages
//! run over the active commits one after the other (so a rule that needs
//! every commit, such as anonymising or "smart hours", sees the result of the
//! rules before it), then the commits are re-created parents first. A commit
//! nothing changed, whose parents did not move, keeps its id and is never
//! re-serialised: its signature survives. Tags and notes follow, and the
//! plan says what is already pushed.

use std::collections::{HashMap, HashSet};
use std::path::Path;

use gitpoulpi_core::domain::rewrite::hashes::{self, Found, HashLookup, HashStats, IdIndex};
use gitpoulpi_core::domain::rewrite::{
    blob_diff as blob_diff_text, validate_recipe, BlobChange, BlobChangeKind, CommitChange,
    CommitterPolicy, ContentFind, ContentReplace, Counts, ExtractMode, FieldChange, PlanDigest,
    PruneEmpty, PushState, Recipe, RefChange, RewriteField, RewritePhase, RewritePlan,
    RewriteProgress, RewriteProgressSink, RewriteRefusal, RewriteWarning, RuleCount, RuleKind,
    RunId, Scope, SignaturePolicy,
};
use gitpoulpi_core::domain::Oid;
use gitpoulpi_core::{Error, Result};
use sha1::{Digest, Sha1};

use super::impact::{self, Pushed};
use super::objects::{Commit, Edit, Person};
use super::pack::{PackReader, Sink};
use super::plumbing;
use super::refs::{self, Active, RefKind, ScopeRef};
use super::rules::{apply_rule, Work};
use super::serialize::{commit_id, object_id};
use super::sign::Signer;
use super::trees::{self, Content};
use super::walk::{self, refuse, Head, Reader};
use super::{blobs, notes, pack, store};

fn canceled() -> Error {
    // The daemon puts its job number on it.
    Error::Canceled { job: 0 }
}

fn is_content_rule(kind: &RuleKind) -> bool {
    !matches!(
        kind,
        RuleKind::Identity(_)
            | RuleKind::Anonymise(_)
            | RuleKind::DateShift { .. }
            | RuleKind::DateTimezone { .. }
            | RuleKind::DateWindow(_)
            | RuleKind::CommitterFromAuthor
            | RuleKind::MessageReplace(_)
            | RuleKind::MessageDeleteLines { .. }
            | RuleKind::Mailmap { .. }
            | RuleKind::Trailer(_)
            | RuleKind::Script(_)
            | RuleKind::RemoveSignatures
            | RuleKind::MessageEncoding { .. }
    )
}

/// What this part of the engine runs; anything else is for a later slice.
fn check_supported(recipe: &Recipe) -> Result<()> {
    for (_, rule) in recipe.enabled_rules() {
        match &rule.kind {
            RuleKind::Script(script) => super::script::supported(script)?,
            RuleKind::ContentReplace(_)
            | RuleKind::RemovePaths { .. }
            | RuleKind::RemoveBlobsOver { .. }
            | RuleKind::LfsImport(_)
            | RuleKind::LfsExport(_)
            | RuleKind::LineEndings(_)
            | RuleKind::StripExec(_)
            | RuleKind::Reencode(_)
            | RuleKind::MovePaths(_) => {}
            kind if is_content_rule(kind) => {
                return Err(Error::Unsupported {
                    operation: "this kind of rewrite rule",
                })
            }
            _ => {}
        }
    }
    Ok(())
}

/// `git var GIT_COMMITTER_IDENT` as a person.
fn me_now(root: &Path) -> Result<Person> {
    Person::parse(&walk::committer_ident(root)?)
        .ok_or_else(|| Error::Backend("git gave no usable committer identity".into()))
}

fn date_text(p: &Option<Person>) -> String {
    p.as_ref().map(Person::date_text).unwrap_or_default()
}

/// The fields a rewritten commit changed, for the plan's rows.
pub(super) fn field_changes(
    w: &Work,
    committer: &Option<Person>,
    was_signed: bool,
    now_signed: bool,
) -> Vec<FieldChange> {
    let mut out = Vec::new();
    let mut push = |field, old: String, new: String| {
        if old != new {
            out.push(FieldChange { field, old, new });
        }
    };
    let name = |p: &Option<Person>| p.as_ref().map(|p| p.name.clone()).unwrap_or_default();
    let email = |p: &Option<Person>| p.as_ref().map(|p| p.email.clone()).unwrap_or_default();
    push(RewriteField::AuthorName, name(&w.author0), name(&w.author));
    push(
        RewriteField::AuthorEmail,
        email(&w.author0),
        email(&w.author),
    );
    push(
        RewriteField::AuthorDate,
        date_text(&w.author0),
        date_text(&w.author),
    );
    push(
        RewriteField::CommitterName,
        name(&w.committer0),
        name(committer),
    );
    push(
        RewriteField::CommitterEmail,
        email(&w.committer0),
        email(committer),
    );
    push(
        RewriteField::CommitterDate,
        date_text(&w.committer0),
        date_text(committer),
    );
    if w.drop_encoding {
        push(
            RewriteField::Encoding,
            w.commit
                .encoding()
                .map(|e| String::from_utf8_lossy(e).into_owned())
                .unwrap_or_default(),
            "UTF-8".to_string(),
        );
    }
    push(
        RewriteField::Message,
        String::from_utf8_lossy(w.commit.message()).into_owned(),
        String::from_utf8_lossy(w.message()).into_owned(),
    );
    if let (Some(new), Some(old)) = (w.tree, w.commit.tree()) {
        out.push(FieldChange {
            field: RewriteField::Tree,
            old: old.to_hex(),
            new: new.to_hex(),
        });
    }
    if was_signed || now_signed {
        out.push(FieldChange {
            field: RewriteField::Signature,
            old: if was_signed { "signed" } else { "" }.into(),
            new: match (was_signed, now_signed) {
                (true, true) => "re-signed",
                (false, true) => "signed",
                _ => "",
            }
            .into(),
        });
    }
    out
}

pub(super) fn digest(recipe: &Recipe, refs: &[RefChange]) -> PlanDigest {
    let mut hasher = Sha1::new();
    // The secret values are not part of it: they are not stored.
    hasher.update(serde_json::to_vec(&recipe.saved()).unwrap_or_default());
    for r in refs {
        hasher.update(format!("\n{} {} ", r.name, r.old.to_hex()));
        hasher.update(r.new.map(|n| n.to_hex()).unwrap_or_default());
    }
    let sum: [u8; 20] = hasher.finalize().into();
    PlanDigest(Oid(sum).to_hex())
}

/// A fresh run id: the clock, the process and a counter, hashed.
fn new_run_id() -> RunId {
    use std::sync::atomic::{AtomicU64, Ordering};
    static COUNTER: AtomicU64 = AtomicU64::new(0);
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |d| d.as_nanos());
    let mut hasher = Sha1::new();
    hasher.update(now.to_le_bytes());
    hasher.update(std::process::id().to_le_bytes());
    hasher.update(COUNTER.fetch_add(1, Ordering::Relaxed).to_le_bytes());
    let sum: [u8; 20] = hasher.finalize().into();
    RunId(format!("run-{}", &Oid(sum).to_hex()[..12]))
}

type Tick<'a> = &'a mut dyn FnMut(RewritePhase, u64, u64) -> Result<()>;

/// What a pass reads.
struct Ctx<'a> {
    root: &'a Path,
    recipe: &'a Recipe,
    order: &'a [Oid],
    commits: &'a [Commit],
    /// Position of each commit of the walk in `order`.
    index: &'a HashMap<Oid, usize>,
    /// Parents as indices into `order`; a parent outside the walk is left out.
    parents: &'a [Vec<usize>],
    active: &'a Active,
    me: Option<&'a Person>,
    pushed: &'a Pushed,
    /// The identities of the annotated tags that may be re-created, for the
    /// rules that rewrite taggers (empty when no rule does).
    taggers: &'a [Person],
    /// What external scripts may do, and the probe that finds interpreters.
    scripts: super::script::ScriptEnv<'a>,
    /// Signs the commits the recipe re-signs (`None`: nothing is signed).
    signer: Option<&'a std::cell::RefCell<Signer>>,
}

/// The commits reachable from `tips` through `member` ones.
fn reach(tips: &[usize], parents: &[Vec<usize>], member: &[bool]) -> Vec<bool> {
    let mut seen = vec![false; parents.len()];
    let mut stack: Vec<usize> = tips.iter().copied().filter(|t| member[*t]).collect();
    for t in &stack {
        seen[*t] = true;
    }
    while let Some(c) = stack.pop() {
        for p in &parents[c] {
            if member[*p] && !seen[*p] {
                seen[*p] = true;
                stack.push(*p);
            }
        }
    }
    seen
}

/// Whether commit `a` is an ancestor of commit `b` (positions in the walk,
/// parents before children).
fn is_ancestor(parents: &[Vec<usize>], a: usize, b: usize) -> bool {
    let mut seen = HashSet::from([b]);
    let mut stack = vec![b];
    while let Some(c) = stack.pop() {
        for p in &parents[c] {
            if *p == a {
                return true;
            }
            if *p > a && seen.insert(*p) {
                stack.push(*p);
            }
        }
    }
    false
}

/// The rules and the re-creation over the `member` commits of the walk.
struct Pass {
    new_id: HashMap<Oid, Oid>,
    created: Vec<(Oid, Vec<u8>)>,
    rows: Vec<CommitChange>,
    per_rule: Vec<RuleCount>,
    signed_changed: u64,
    /// Commits this pass signed (cached signatures included).
    resigned: u64,
    walked: u64,
    /// Tagger identities the rules changed: `(name, email)` before and after.
    taggers: HashMap<(String, String), (String, String)>,
    /// Commits dropped because a content rule emptied them; `new_id` maps
    /// each to the commit its children now sit on.
    dropped: HashSet<Oid>,
    /// Commits that are not in the new history at all (an extraction of a
    /// directory they do not have): no commit stands in for them, their
    /// children lose the parent. A subset of `dropped`.
    vanished: HashSet<Oid>,
    /// Old commits whose tree changed (dropped ones included).
    tree_moved: HashSet<Oid>,
    /// `update_message_hashes`: what happened to the ids in messages.
    hashes: HashStats,
    /// The message-encoding rule: commits it could not decide on.
    encoding_left: u64,
}

/// The tree of commit `id`, from the walk when it is in it.
fn tree_of(ctx: &Ctx<'_>, reader: &mut Reader, id: Oid) -> Result<Option<Oid>> {
    if let Some(i) = ctx.index.get(&id) {
        return Ok(ctx.commits[*i].tree());
    }
    let (_, raw) = reader.get(id)?;
    Ok(Commit::parse(raw)?.tree())
}

fn run_pass(
    ctx: &Ctx<'_>,
    member: &[bool],
    tick: Tick<'_>,
    reader: &mut Reader,
    content: &mut Option<Content>,
) -> Result<Pass> {
    let subset: Vec<usize> = (0..ctx.order.len()).filter(|i| member[*i]).collect();
    let total = subset.len() as u64;
    let mut works: Vec<Work> = subset
        .iter()
        .map(|i| Work::new(ctx.commits[*i].clone()))
        .collect();
    let sub_order: Vec<Oid> = subset.iter().map(|i| ctx.order[*i]).collect();
    let active: Vec<bool> = match ctx.active {
        Active::All => vec![true; subset.len()],
        Active::Set(set) => sub_order.iter().map(|o| set.contains(o)).collect(),
        Active::UpTo(tip) => {
            let Some(t) = ctx
                .order
                .iter()
                .position(|o| o == tip)
                .filter(|t| member[*t])
            else {
                return Err(refuse(
                    RewriteRefusal::EmptyScope,
                    "the range is not in the history of the current branch",
                ));
            };
            let seen = reach(&[t], ctx.parents, member);
            subset.iter().map(|i| seen[*i]).collect()
        }
    };
    if !active.iter().any(|a| *a) {
        return Err(refuse(
            RewriteRefusal::EmptyScope,
            "no selected commit is in the history to rewrite",
        ));
    }

    let mut per_rule = Vec::new();
    let mut encoding_left = 0u64;
    let mut taggers: Vec<Person> = ctx.taggers.to_vec();
    let strip_rule = ctx
        .recipe
        .enabled_rules()
        .any(|(_, r)| matches!(r.kind, RuleKind::RemoveSignatures));
    for (index, rule) in ctx.recipe.enabled_rules() {
        if is_content_rule(&rule.kind) {
            continue;
        }
        if matches!(rule.kind, RuleKind::RemoveSignatures) {
            let n = works
                .iter()
                .zip(&active)
                .filter(|(w, a)| **a && w.commit.is_signed())
                .count() as u64;
            per_rule.push(RuleCount {
                rule: index,
                commits: n,
            });
            continue;
        }
        tick(RewritePhase::Rewriting, 0, total)?;
        if let RuleKind::Script(script_ref) = &rule.kind {
            // Fresh state per pass: a script's `ctx.state` starts empty.
            let script = super::script::load(ctx.root, script_ref, ctx.scripts)?;
            let n = super::script::run_stage(
                &script,
                &mut works,
                &sub_order,
                &active,
                &mut |done| tick(RewritePhase::Rewriting, done, total),
                &mut |_, _| {},
            )?;
            per_rule.push(RuleCount {
                rule: index,
                commits: n,
            });
            continue;
        }
        let (mut chosen, chosen_order): (Vec<&mut Work>, Vec<Oid>) = works
            .iter_mut()
            .zip(&sub_order)
            .zip(&active)
            .filter(|(_, a)| **a)
            .map(|((w, o), _)| (w, *o))
            .unzip();
        let n = match &rule.kind {
            RuleKind::Mailmap { apply_to, file } => {
                let map = super::rules::load_mailmap(ctx.root, file.as_deref())?;
                super::rules::apply_mailmap(&mut chosen, &map, *apply_to)
            }
            RuleKind::MessageEncoding { names } => {
                let (fixed, left) = super::encoding::stage(&mut chosen, *names);
                encoding_left += left;
                fixed
            }
            kind => apply_rule(&mut chosen, &chosen_order, kind, &mut taggers)?,
        };
        per_rule.push(RuleCount {
            rule: index,
            commits: n,
        });
    }

    // The content rules: each active commit's tree, rewritten once per
    // unique tree and blob for the whole run.
    let mut tree_moved: HashSet<Oid> = HashSet::new();
    let mut emptied = vec![false; works.len()];
    if let Some(c) = content.as_mut() {
        let mut hits = vec![0u64; c.rule_ids.len()];
        let mut collisions: Vec<String> = Vec::new();
        for (k, w) in works.iter_mut().enumerate() {
            if k % 64 == 0 {
                tick(RewritePhase::Rewriting, k as u64, total)?;
            }
            if !active[k] {
                continue;
            }
            let Some(old_tree) = w.commit.tree() else {
                continue;
            };
            let done = c.rewrite_root(reader, old_tree)?;
            if let Some(path) = &done.collision {
                let id = sub_order[k].to_hex();
                collisions.push(format!("{}: {path}", &id[..7]));
            }
            emptied[k] = done.vanished_root;
            if done.changed {
                w.tree = done.new;
                w.paths = done.paths.clone();
                for (n, count) in hits.iter_mut().enumerate() {
                    *count += u64::from(done.hit & trees::bit(n) != 0);
                }
            }
        }
        if !collisions.is_empty() {
            const LISTED: usize = 50;
            let more = collisions.len().saturating_sub(LISTED);
            collisions.truncate(LISTED);
            let mut detail = format!(
                "the target path already exists in {} commit(s); the commit and the path:\n{}",
                collisions.len() + more,
                collisions.join("\n")
            );
            if more > 0 {
                detail.push_str("\n...");
            }
            return Err(refuse(RewriteRefusal::PathCollision, &detail));
        }
        for (n, commits) in hits.into_iter().enumerate() {
            per_rule.push(RuleCount {
                rule: c.rule_ids[n],
                commits,
            });
        }
        per_rule.sort_by_key(|r| r.rule);
    }
    let prune = content.is_some() && ctx.recipe.options.prune_empty == PruneEmpty::Auto;

    // Re-create the commits, parents first.
    let mut new_id: HashMap<Oid, Oid> = HashMap::new();
    let mut new_tree: HashMap<Oid, Oid> = HashMap::new();
    let mut dropped: HashSet<Oid> = HashSet::new();
    let mut vanished: HashSet<Oid> = HashSet::new();
    // Where in the walk each commit this pass created sits.
    let mut origin: HashMap<Oid, usize> = HashMap::new();
    let extracts = content.as_ref().is_some_and(|c| c.extracts);
    let mut created: Vec<(Oid, Vec<u8>)> = Vec::new();
    let mut rows = Vec::new();
    let mut signed_changed = 0u64;
    let mut resigned = 0u64;
    let policy = ctx.recipe.options.signatures;
    // An upper bound for the progress: the commits that could be signed.
    let sign_total = match (ctx.signer, policy) {
        (None, _) => 0,
        (Some(_), Some(SignaturePolicy::ResignAll)) => total,
        (Some(_), _) => works.iter().filter(|w| w.commit.is_signed()).count() as u64,
    };
    let mut hash_stats = HashStats::default();
    let hash_index = ctx
        .recipe
        .options
        .update_message_hashes
        .then(|| IdIndex::new(sub_order.iter().copied()));
    for (i, (old, w)) in sub_order.iter().zip(works.iter_mut()).enumerate() {
        if i % 256 == 0 {
            tick(RewritePhase::Rewriting, i as u64, total)?;
        }
        if let (Some(index), true) = (&hash_index, active[i]) {
            if let Ok(text) = std::str::from_utf8(w.message()) {
                let (text, stats) = hashes::update(text, |prefix| match index.find(prefix) {
                    Found::One(j) if j < i => new_id
                        .get(&sub_order[j])
                        .map_or(HashLookup::Same, |n| HashLookup::New(n.to_hex())),
                    Found::One(j) if j == i => HashLookup::Same,
                    // Created later: its new id is not known yet.
                    Found::One(_) | Found::Many => HashLookup::Ambiguous,
                    Found::None => HashLookup::Unknown,
                });
                hash_stats += stats;
                if stats.replaced > 0 {
                    w.message = Some(text.into_bytes());
                }
            }
        }
        let w: &Work = w;
        let old_parents = w.commit.parents()?;
        let mut parents: Vec<Oid> = old_parents
            .iter()
            .filter(|p| !vanished.contains(p))
            .map(|p| *new_id.get(p).unwrap_or(p))
            .collect();
        // Two parents that became the same commit are one parent.
        let mut seen_parents = HashSet::new();
        parents.retain(|p| seen_parents.insert(*p));
        // An extraction also drops the parent of a merge that is an
        // ancestor of another one: the side branch that touched nothing of
        // the directory leaves only an old commit behind.
        if extracts && parents.len() > 1 {
            let at: Vec<Option<usize>> = parents
                .iter()
                .map(|p| origin.get(p).or_else(|| ctx.index.get(p)).copied())
                .collect();
            if at.iter().all(Option::is_some) {
                let at: Vec<usize> = at.into_iter().flatten().collect();
                let keep: Vec<bool> = (0..at.len())
                    .map(|k| {
                        !(0..at.len()).any(|j| j != k && is_ancestor(ctx.parents, at[k], at[j]))
                    })
                    .collect();
                let mut flags = keep.iter();
                parents.retain(|_| *flags.next().unwrap_or(&true));
            }
        }
        let author_changed = w.author != w.author0;
        let committer_changed = w.committer != w.committer0;
        let message_changed = w
            .message
            .as_deref()
            .is_some_and(|m| m != w.commit.message());
        let tree_changed = w.tree.is_some();
        if tree_changed {
            tree_moved.insert(*old);
        }
        // "Remove signatures" changes a signed commit by itself.
        let stripped_by_rule = strip_rule && active[i] && w.commit.is_signed();
        if !(author_changed
            || committer_changed
            || message_changed
            || w.drop_encoding
            || tree_changed
            || stripped_by_rule
            || parents != old_parents)
        {
            continue;
        }
        // An extraction of a directory the commit does not have, with no
        // parent that has it either: the commit is not in the new history.
        if extracts && emptied[i] && parents.is_empty() {
            vanished.insert(*old);
            dropped.insert(*old);
            rows.push(CommitChange {
                old: *old,
                new: None,
                fields: Vec::new(),
                paths: w.paths.to_vec(),
                pushed: ctx.pushed.is_pushed(old),
            });
            continue;
        }
        // A commit the content rules emptied goes, and its children sit on
        // its (new) parent; a merge whose parents collapsed onto one commit
        // that has its tree goes the same way. One that was empty before
        // stays.
        if let (true, true, Some(tree)) = (prune, tree_changed, w.tree) {
            if let Some(first) = parents.first().copied() {
                let same_as_parent = match new_tree.get(&first) {
                    Some(t) => *t == tree,
                    None => tree_of(ctx, reader, first)? == Some(tree),
                };
                let empty_before = match old_parents.as_slice() {
                    [only] => tree_of(ctx, reader, *only)? == w.commit.tree(),
                    _ => false,
                };
                let collapsed = parents.len() == 1;
                if same_as_parent && !empty_before && collapsed {
                    new_id.insert(*old, first);
                    dropped.insert(*old);
                    rows.push(CommitChange {
                        old: *old,
                        new: None,
                        fields: Vec::new(),
                        paths: w.paths.to_vec(),
                        pushed: ctx.pushed.is_pushed(old),
                    });
                    continue;
                }
            }
        }
        // "Me, now" stamps every commit that is re-created anyway.
        let committer = match (ctx.me, &w.committer) {
            (Some(me), Some(_)) => Some(me.clone()),
            _ => w.committer.clone(),
        };
        let committer_edited = committer != w.committer0;
        let body = w.commit.serialise(&Edit {
            tree: w.tree,
            parents: &parents,
            author: if author_changed {
                w.author.as_ref()
            } else {
                None
            },
            committer: if committer_edited {
                committer.as_ref()
            } else {
                None
            },
            message: w.message.as_deref().filter(|_| message_changed),
            drop_encoding: w.drop_encoding,
        });
        let signed = w.commit.is_signed();
        signed_changed += u64::from(signed && !stripped_by_rule);
        let sign_now = !(strip_rule && active[i])
            && match policy {
                Some(SignaturePolicy::ResignSigned) => signed,
                Some(SignaturePolicy::ResignAll) => true,
                _ => false,
            };
        let body = match (sign_now, ctx.signer) {
            (true, Some(signer)) => {
                tick(
                    RewritePhase::Signing,
                    resigned,
                    sign_total.max(resigned + 1),
                )?;
                let sig = signer
                    .borrow_mut()
                    .sign(&body)
                    .map_err(|f| refuse(f.reason, &f.detail))?;
                resigned += 1;
                super::sign::attach(&body, &sig)
            }
            _ => body,
        };
        let id = commit_id(&body);
        rows.push(CommitChange {
            old: *old,
            new: Some(id),
            fields: field_changes(w, &committer, signed, sign_now && ctx.signer.is_some()),
            paths: w.paths.to_vec(),
            pushed: ctx.pushed.is_pushed(old),
        });
        new_id.insert(*old, id);
        origin.insert(id, subset[i]);
        if let Some(tree) = w.tree.or_else(|| w.commit.tree()) {
            new_tree.insert(id, tree);
        }
        created.push((id, body));
    }
    let taggers = ctx
        .taggers
        .iter()
        .zip(&taggers)
        .filter(|(old, new)| (&old.name, &old.email) != (&new.name, &new.email))
        .map(|(old, new)| {
            (
                (old.name.clone(), old.email.clone()),
                (new.name.clone(), new.email.clone()),
            )
        })
        .collect();
    Ok(Pass {
        new_id,
        created,
        rows,
        per_rule,
        signed_changed,
        resigned,
        walked: total,
        taggers,
        dropped,
        vanished,
        tree_moved,
        hashes: hash_stats,
        encoding_left,
    })
}

/// The commit an annotated tag finally points at.
fn peel(reader: &mut Reader, mut oid: Oid) -> Option<Oid> {
    for _ in 0..16 {
        let (kind, body) = reader.get(oid).ok()?;
        match kind.as_str() {
            "commit" => return Some(oid),
            "tag" => oid = plumbing::tag_target(&body)?.object,
            _ => return None,
        }
    }
    None
}

fn rewrites_taggers(kind: &RuleKind) -> bool {
    match kind {
        RuleKind::Identity(rule) => rule.include_taggers,
        RuleKind::Anonymise(rule) => rule.include_taggers,
        _ => false,
    }
}

/// The distinct taggers of the annotated tags that reach a commit (a tag of a
/// tag counts both), in tag name order. More than the tags a run will
/// re-create: a tagger nobody uses is only a number skipped.
fn tagger_identities(reader: &mut Reader, tags: &[refs::RefLine]) -> Result<Vec<Person>> {
    let mut out: Vec<Person> = Vec::new();
    let mut sorted: Vec<&refs::RefLine> = tags.iter().filter(|t| t.reaches_commit()).collect();
    sorted.sort_by(|a, b| a.name.cmp(&b.name));
    for t in sorted {
        let (mut kind, mut oid) = (t.kind.clone(), t.oid);
        for _ in 0..16 {
            if kind != "tag" {
                break;
            }
            let (_, body) = reader.get(oid)?;
            if let Some(p) = plumbing::tagger_of(&body) {
                if !out
                    .iter()
                    .any(|o| (&o.name, &o.email) == (&p.name, &p.email))
                {
                    out.push(p);
                }
            }
            let Some(target) = plumbing::tag_target(&body) else {
                break;
            };
            (kind, oid) = (target.kind, target.object);
        }
    }
    Ok(out)
}

/// What re-creating annotated tags needs and leaves behind.
struct Retag<'a> {
    new_id: &'a HashMap<Oid, Oid>,
    /// Tag objects already handled: the new id, or `None` when unchanged.
    memo: HashMap<Oid, Option<Oid>>,
    /// The new tag objects, for the pack.
    objects: Vec<(u8, Vec<u8>)>,
    /// New tags that lost a signature.
    signed_dropped: u32,
    /// Tagger identities the rules changed (see `Pass::taggers`).
    taggers: &'a HashMap<(String, String), (String, String)>,
}

impl Retag<'_> {
    /// The id `oid` becomes: a rewritten commit's new id, or a tag object
    /// re-created on its (new) target. `None` when nothing it points at moved.
    fn follow(&mut self, reader: &mut Reader, kind: &str, oid: Oid) -> Result<Option<Oid>> {
        if kind == "commit" {
            return Ok(self.new_id.get(&oid).copied());
        }
        if kind != "tag" {
            return Ok(None);
        }
        if let Some(done) = self.memo.get(&oid) {
            return Ok(*done);
        }
        let (_, body) = reader.get(oid)?;
        let new = match plumbing::tag_target(&body) {
            Some(target) => match self.follow(reader, &target.kind, target.object)? {
                Some(new_object) => {
                    let tagger = plumbing::tagger_of(&body).and_then(|mut p| {
                        let (name, email) = self.taggers.get(&(p.name.clone(), p.email.clone()))?;
                        p.name.clone_from(name);
                        p.email.clone_from(email);
                        Some(p)
                    });
                    let (new_body, signed) =
                        plumbing::retarget_tag(&body, new_object, tagger.as_ref());
                    let id = object_id("tag", &new_body);
                    self.objects.push((pack::TAG, new_body));
                    self.signed_dropped += u32::from(signed);
                    Some(id)
                }
                None => None,
            },
            None => None,
        };
        self.memo.insert(oid, new);
        Ok(new)
    }
}

pub(crate) fn preview(
    root: &Path,
    recipe: &Recipe,
    tools: &crate::tools::ToolProbe,
    sink: RewriteProgressSink<'_>,
) -> Result<RewritePlan> {
    validate_recipe(recipe)
        .map_err(|issue| Error::RewriteRefused(issue.refusal, issue.to_string()))?;
    let mut tick = |phase, done, total| -> Result<()> {
        match sink(RewriteProgress { phase, done, total }) {
            std::ops::ControlFlow::Continue(()) => Ok(()),
            std::ops::ControlFlow::Break(()) => Err(canceled()),
        }
    };
    tick(RewritePhase::Walking, 0, 0)?;
    let probe = walk::probe(root)?;
    if probe.partial_clone
        && recipe
            .enabled_rules()
            .any(|(_, r)| is_content_rule(&r.kind))
    {
        return Err(refuse(
            RewriteRefusal::PartialClone,
            "a partial clone would fetch file content one object at a time",
        ));
    }
    check_supported(recipe)?;
    // An extraction that leaves this repository alone (a new clone, a
    // submodule) is previewed over every local branch and tag: that is what
    // the new repository gets. Nothing of it is pushed or protected here.
    let elsewhere = matches!(
        recipe.extraction(),
        Some((_, Some(ExtractMode::Clone | ExtractMode::Submodule)))
    );
    let widened;
    let recipe = if elsewhere {
        let mut r = recipe.clone();
        r.scope = Scope::AllLocal;
        r.scope_options.include_tags = true;
        widened = r;
        &widened
    } else {
        recipe
    };
    // A re-sign needs a signing setup that works: found out now, before the
    // history is walked, with a typed reason.
    let signer = if matches!(
        recipe.options.signatures,
        Some(SignaturePolicy::ResignSigned | SignaturePolicy::ResignAll)
    ) && !recipe
        .enabled_rules()
        .any(|(_, r)| matches!(r.kind, RuleKind::RemoveSignatures))
    {
        let signer = Signer::open_checked(root, tools).map_err(|f| refuse(f.reason, &f.detail))?;
        Some(std::cell::RefCell::new(signer))
    } else {
        None
    };
    let lfs_run = recipe
        .enabled_rules()
        .any(|(_, r)| matches!(r.kind, RuleKind::LfsImport(_) | RuleKind::LfsExport(_)));
    if lfs_run {
        // The result needs git-lfs to check out, so the run needs it.
        let binary = crate::tools::lfs_of_status(
            &tools.get(gitpoulpi_core::domain::rewrite::Tool::GitLfs, root),
        );
        if let Some(reason) = binary.unavailable() {
            return Err(refuse(RewriteRefusal::ToolMissing, &reason.message()));
        }
    }
    let scripts = super::script::ScriptEnv {
        policy: &recipe.script_policy,
        probe: tools,
    };
    // A script that does not load fails now, before the history is walked.
    for (_, rule) in recipe.enabled_rules() {
        if let RuleKind::Script(script) = &rule.kind {
            super::script::load(root, script, scripts)?;
        }
    }
    // The run's directory: removed again unless the preview ends as a plan
    // Apply can use (also on a refusal or a cancel).
    let run = new_run_id();
    let dir = store::run_dir(&probe.git_dir, &run);
    let mut guard = RunGuard {
        dir: dir.clone(),
        armed: true,
    };
    // A secret rule that scans names findings: the values come from a scan
    // of the same scope, held here for this preview only.
    let found = super::scan::resolve(root, recipe, &mut tick)?;
    let mut content = Content::with_secrets(recipe, Sink::new(dir.clone()), &found)?;
    if let (Some(c), true) = (content.as_mut(), lfs_run) {
        c.set_lfs_storage(super::lfs::storage_dir(root, &probe.common_dir));
    }
    let all_refs = refs::list(root)?;
    let resolved = refs::resolve(&recipe.scope, &recipe.scope_options, &all_refs, &probe.head)?;
    let pushed = if elsewhere {
        Pushed::none()
    } else {
        impact::classify(root, &all_refs, &probe.head)?
    };
    let order = walk::rev_list(root, &resolved.tips, &resolved.not)?;
    if order.is_empty() {
        return Err(refuse(
            RewriteRefusal::EmptyScope,
            "the scope has no commit",
        ));
    }
    let total = order.len() as u64;
    tick(RewritePhase::Walking, 0, total)?;
    let mut reader = Reader::new(root)?;
    let raw = reader.commits(&order, |i| tick(RewritePhase::Walking, i, total))?;
    let mut commits = Vec::with_capacity(raw.len());
    for bytes in raw {
        commits.push(Commit::parse(bytes)?);
    }
    let index: HashMap<Oid, usize> = order.iter().enumerate().map(|(i, o)| (*o, i)).collect();
    let mut parents: Vec<Vec<usize>> = Vec::with_capacity(commits.len());
    for c in &commits {
        parents.push(
            c.parents()?
                .iter()
                .filter_map(|p| index.get(p).copied())
                .collect(),
        );
    }
    let me = match recipe.options.committer {
        CommitterPolicy::MeNow => Some(me_now(root)?),
        CommitterPolicy::Keep => None,
    };
    let taggers = if recipe
        .enabled_rules()
        .any(|(_, r)| rewrites_taggers(&r.kind))
    {
        tagger_identities(&mut reader, &all_refs.tags)?
    } else {
        Vec::new()
    };
    let ctx = Ctx {
        root,
        recipe,
        order: &order,
        commits: &commits,
        index: &index,
        parents: &parents,
        active: &resolved.active,
        me: me.as_ref(),
        pushed: &pushed,
        taggers: &taggers,
        scripts,
        signer: signer.as_ref(),
    };

    let mut member = vec![true; order.len()];
    let mut pass = run_pass(&ctx, &member, &mut tick, &mut reader, &mut content)?;

    // Protected branches that a pushed, rewritten commit sits on are left out
    // unless the caller lists them; the rest is replayed without them.
    let protected = refs::protected_branches(
        root,
        &probe.common_dir,
        &recipe.scope_options.protected_branches,
    );
    let full = |n: &String| {
        if n.starts_with("refs/heads/") {
            n.clone()
        } else {
            format!("refs/heads/{n}")
        }
    };
    let included: HashSet<String> = recipe
        .scope_options
        .include_protected
        .iter()
        .map(full)
        .collect();
    let tip_of = |r: &ScopeRef, reader: &mut Reader| -> Option<usize> {
        match r.kind {
            RefKind::Tag => peel(reader, r.old).and_then(|c| index.get(&c).copied()),
            _ => index.get(&r.old).copied(),
        }
    };
    let mut in_scope = resolved.in_scope.clone();
    let mut excluded: Vec<String> = Vec::new();
    for r in &in_scope {
        if elsewhere
            || r.kind != RefKind::Branch
            || !protected.contains(&r.name)
            || included.contains(&r.name)
        {
            continue;
        }
        let Some(tip) = tip_of(r, &mut reader) else {
            continue;
        };
        let seen = reach(&[tip], &parents, &member);
        let hit = (0..order.len())
            .any(|i| seen[i] && pass.new_id.contains_key(&order[i]) && pushed.is_pushed(&order[i]));
        if hit {
            excluded.push(r.name.clone());
        }
    }
    if !excluded.is_empty() {
        excluded.sort();
        in_scope.retain(|r| !excluded.contains(&r.name));
        if in_scope.is_empty() {
            return Err(refuse(
                RewriteRefusal::ProtectedRef,
                &format!(
                    "{} is protected and already pushed; list it in include_protected to rewrite it",
                    excluded.join(", ")
                ),
            ));
        }
        let tips: Vec<usize> = in_scope
            .iter()
            .filter_map(|r| tip_of(r, &mut reader))
            .collect();
        member = reach(&tips, &parents, &vec![true; order.len()]);
        if !member.iter().any(|m| *m) {
            return Err(refuse(
                RewriteRefusal::EmptyScope,
                "the scope has no commit left",
            ));
        }
        pass = run_pass(&ctx, &member, &mut tick, &mut reader, &mut content)?;
    }
    if pass.signed_changed > 0 && recipe.options.signatures.is_none() {
        return Err(refuse(
            RewriteRefusal::SignatureChoiceNeeded,
            &format!(
                "{} signed commits would be rewritten; choose what happens to their signatures",
                pass.signed_changed
            ),
        ));
    }

    if let Some(c) = &content {
        if !c.lfs().missing().is_empty() {
            return Err(refuse(
                RewriteRefusal::LfsObjectsMissing,
                &super::lfs::missing_detail(c.lfs().missing()),
            ));
        }
    }

    tick(RewritePhase::Rewriting, total, total)?;
    let worktrees = if elsewhere {
        Default::default()
    } else {
        refs::other_worktrees(&probe.common_dir, &probe.git_dir)
    };
    let mut changes: Vec<RefChange> = Vec::new();
    for r in in_scope.iter().filter(|r| r.kind != RefKind::Tag) {
        // `None`: the branch has no commit left in the new history.
        let new = match pass.new_id.get(&r.old) {
            Some(new) => Some(*new),
            None if pass.vanished.contains(&r.old) => None,
            None => continue,
        };
        let on_remote = index.get(&r.old).map_or(0, |tip| {
            let seen = reach(&[*tip], &parents, &member);
            (0..order.len())
                .filter(|i| {
                    seen[*i] && pass.new_id.contains_key(&order[*i]) && pushed.is_pushed(&order[*i])
                })
                .count() as u32
        });
        changes.push(RefChange {
            name: r.name.clone(),
            old: r.old,
            new,
            pushed: if elsewhere {
                PushState::LocalOnly
            } else {
                impact::branch_state(&all_refs, &r.name, on_remote)
            },
            protected: !elsewhere && protected.contains(&r.name),
            included: true,
            checked_out_in: worktrees.get(&r.name).cloned(),
        });
    }
    let branches_moved = changes.len() as u32;

    // The checked-out commit's files: a branch checked out in a linked
    // worktree cannot be updated from here, and local edits to a file the
    // rewrite changes would be lost.
    let head_name = match &probe.head {
        Head::Branch(name) => name.clone(),
        Head::Detached(_) => "HEAD".to_string(),
    };
    let mut head_tree_changed = false;
    if let Some(gone) = changes
        .iter()
        .find(|r| r.name == head_name && r.new.is_none())
    {
        return Err(refuse(
            RewriteRefusal::EmptyScope,
            &format!(
                "the directory is not in the history of {}, nothing would be left of it",
                gone.name
            ),
        ));
    }
    if let (false, Some(c)) = (elsewhere, content.as_mut()) {
        if let Some(other) = changes
            .iter()
            .find(|r| r.checked_out_in.is_some() && pass.tree_moved.contains(&r.old))
        {
            return Err(refuse(
                RewriteRefusal::CheckedOutElsewhere,
                &format!(
                    "{} is checked out in {} and its files would change",
                    other.name,
                    other
                        .checked_out_in
                        .as_ref()
                        .map(|p| p.display().to_string())
                        .unwrap_or_default()
                ),
            ));
        }
        if let Some(head) = changes
            .iter()
            .find(|r| r.name == head_name && pass.tree_moved.contains(&r.old))
        {
            head_tree_changed = true;
            let tree = tree_of(&ctx, &mut reader, head.old)?;
            refuse_dirty(root, c, &mut reader, tree)?;
        }
    }

    // Tags: the ones the scope names, and (with `include_tags`) the ones that
    // point at a rewritten commit.
    let named: HashSet<&str> = in_scope
        .iter()
        .filter(|r| r.kind == RefKind::Tag)
        .map(|r| r.name.as_str())
        .collect();
    let mut tag_lines: Vec<&refs::RefLine> = all_refs
        .tags
        .iter()
        .filter(|t| t.reaches_commit() && (named.contains(t.name.as_str()) || resolved.tags_follow))
        .collect();
    tag_lines.sort_by(|a, b| a.name.cmp(&b.name));
    let mut retag = Retag {
        new_id: &pass.new_id,
        memo: HashMap::new(),
        objects: Vec::new(),
        signed_dropped: 0,
        taggers: &pass.taggers,
    };
    for t in tag_lines {
        // A tag on a commit that is not in the new history goes with it.
        let target = if t.kind == "commit" {
            Some(t.oid)
        } else {
            peel(&mut reader, t.oid)
        };
        let new = if target.is_some_and(|c| pass.vanished.contains(&c)) {
            None
        } else {
            match retag.follow(&mut reader, &t.kind, t.oid)? {
                Some(new) => Some(new),
                None => continue,
            }
        };
        changes.push(RefChange {
            name: t.name.clone(),
            old: t.oid,
            new,
            pushed: if elsewhere {
                PushState::LocalOnly
            } else {
                impact::tag_state(&all_refs, &t.name)
            },
            protected: false,
            included: true,
            checked_out_in: None,
        });
    }
    let tags_moved = changes.len() as u32 - branches_moved;
    let Retag {
        mut objects,
        signed_dropped,
        ..
    } = retag;

    // Notes ride along.
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |d| d.as_secs() as i64);
    // A note on a dropped commit has nowhere to go: it is not carried over
    // to the parent, which may have a note of its own.
    let mut note_ids = pass.new_id.clone();
    note_ids.retain(|old, _| !pass.dropped.contains(old));
    for moved in notes::remap(&mut reader, &all_refs.notes, &note_ids, now, &mut objects)? {
        changes.push(RefChange {
            name: moved.name,
            old: moved.old,
            new: Some(moved.new),
            pushed: PushState::LocalOnly,
            protected: false,
            included: true,
            checked_out_in: None,
        });
    }

    // What is left behind.
    let mut warnings = Vec::new();
    if pass.signed_changed > 0 && recipe.options.signatures == Some(SignaturePolicy::Strip) {
        warnings.push(RewriteWarning::SignaturesAffected {
            count: pass.signed_changed,
        });
    }
    let on_remote = changes
        .iter()
        .filter(|c| matches!(c.pushed, PushState::OnRemote { .. }))
        .count() as u32;
    if on_remote > 0 {
        warnings.push(RewriteWarning::PushedRefs { count: on_remote });
    }
    if signed_dropped > 0 {
        warnings.push(RewriteWarning::SignedTagsDropped {
            count: signed_dropped,
        });
    }
    if !excluded.is_empty() {
        warnings.push(RewriteWarning::ProtectedExcluded {
            refs: excluded.clone(),
        });
    }
    if !pass.new_id.is_empty() && !elsewhere {
        let moved_names: HashSet<&str> = changes.iter().map(|c| c.name.as_str()).collect();
        let candidates: HashSet<String> = all_refs
            .heads
            .iter()
            .map(|h| h.name.clone())
            .filter(|n| !moved_names.contains(n.as_str()))
            .collect();
        let roots: Vec<Oid> = (0..order.len())
            .filter(|i| {
                pass.new_id.contains_key(&order[*i])
                    && parents[*i]
                        .iter()
                        .all(|p| !pass.new_id.contains_key(&order[*p]))
            })
            .map(|i| order[i])
            .collect();
        let kept = impact::branches_containing(root, &roots, &candidates)?;
        if !kept.is_empty() {
            warnings.push(RewriteWarning::OtherBranchesKeepOldHistory { refs: kept });
        }
        let rewritten: HashSet<Oid> = pass.new_id.keys().copied().collect();
        let stashes = impact::stashes_on(&probe.common_dir, &mut reader, &rewritten);
        if stashes > 0 {
            warnings.push(RewriteWarning::StashesKeepOldHistory { count: stashes });
        }
    }
    drop(reader);

    let counters = content.as_ref().map(Content::counters);
    let plan = RewritePlan {
        run: run.clone(),
        digest: digest(recipe, &changes),
        counts: Counts {
            commits_walked: pass.walked,
            commits_changed: pass.created.len() as u64,
            commits_dropped: pass.dropped.len() as u64,
            refs_moved: branches_moved,
            tags_moved,
            signed_changed: pass.signed_changed,
            resigned: pass.resigned,
            signing_millis: signer.as_ref().map_or(0, |s| s.borrow().millis),
            blobs_changed: counters.as_ref().map_or(0, |c| c.blobs_changed),
            blobs_skipped_binary: counters.as_ref().map_or(0, |c| c.skipped_binary),
            blobs_skipped_large: counters.as_ref().map_or(0, |c| c.skipped_large),
            blobs_skipped_undecodable: counters.as_ref().map_or(0, |c| c.skipped_undecodable),
            head_tree_changed,
            message_hashes_updated: pass.hashes.replaced,
            message_hashes_left: pass.hashes.left,
            lfs_blobs: counters.as_ref().map_or(0, |c| c.lfs_blobs),
            lfs_bytes: counters.as_ref().map_or(0, |c| c.lfs_bytes),
            lfs_objects: counters.as_ref().map_or(0, |c| c.lfs_objects),
            lfs_restored: counters.as_ref().map_or(0, |c| c.lfs_restored),
            lfs_run,
            encoding_left: pass.encoding_left,
        },
        refs: changes,
        warnings,
        per_rule: pass.per_rule,
        rows: pass.rows,
    };
    if plan.refs.is_empty() && pass.created.is_empty() {
        return Ok(plan);
    }

    tick(RewritePhase::Indexing, 0, total)?;
    let mut sink = match content {
        Some(c) => c.sink,
        None => Sink::new(dir.clone()),
    };
    for (_, body) in &pass.created {
        sink.add(pack::COMMIT, body)?;
    }
    for (kind, body) in &objects {
        sink.add(*kind, body)?;
    }
    sink.finish()?;
    store::write_commit_map(&dir, &order, &pass.new_id, &pass.dropped)?;
    store::write_plan(&dir, &plan, recipe)?;
    guard.armed = false;
    Ok(plan)
}

/// Removes the run directory when the preview does not end as a plan.
struct RunGuard {
    dir: std::path::PathBuf,
    armed: bool,
}

impl Drop for RunGuard {
    fn drop(&mut self) {
        if self.armed {
            let _ = std::fs::remove_dir_all(&self.dir);
        }
    }
}

/// Refuses when a file with local changes (staged or not) is one the rules
/// would change or remove in the checked-out tree `head`.
fn refuse_dirty(
    root: &Path,
    content: &mut Content,
    reader: &mut Reader,
    head: Option<Oid>,
) -> Result<()> {
    let Some(head) = head else { return Ok(()) };
    let out = crate::process::git(
        root,
        &["status", "--porcelain=v1", "-z", "--no-renames", "-uno"],
    )
    .env("GIT_OPTIONAL_LOCKS", "0")
    .output()
    .map_err(crate::process::spawn_failed)?;
    if !out.status.success() {
        return Err(Error::Backend(format!(
            "git status failed: {}",
            String::from_utf8_lossy(&out.stderr).trim()
        )));
    }
    // Every such file, so the workbench can list what is in the way (capped).
    let mut dirty: Vec<String> = Vec::new();
    for record in out.stdout.split(|b| *b == 0).filter(|r| r.len() > 3) {
        let path = String::from_utf8_lossy(&record[3..]).into_owned();
        if content.would_change(reader, head, &path)? {
            dirty.push(path);
        }
    }
    if !dirty.is_empty() {
        const LISTED: usize = 50;
        let more = dirty.len().saturating_sub(LISTED);
        dirty.truncate(LISTED);
        let mut detail = dirty.join("\n");
        if more > 0 {
            detail.push_str("\n...");
        }
        return Err(refuse(RewriteRefusal::DirtyPathsTouched, &detail));
    }
    Ok(())
}

/// The file at `path` in `tree`: trees and the blob come from the preview's
/// pack when it has them (new objects), else from the repository.
fn file_in(
    reader: &mut Reader,
    pack: Option<&PackReader>,
    tree: Oid,
    path: &str,
) -> Result<Option<Vec<u8>>> {
    let body_of = |reader: &mut Reader, id: Oid| -> Result<Vec<u8>> {
        match pack.and_then(|p| p.get(id)) {
            Some(body) => Ok(body),
            None => Ok(reader.get(id)?.1),
        }
    };
    let parts: Vec<&str> = path.split('/').collect();
    let mut at = tree;
    for (i, part) in parts.iter().enumerate() {
        let entries = plumbing::parse_tree(&body_of(reader, at)?).unwrap_or_default();
        let Some(entry) = entries.iter().find(|e| e.name == part.as_bytes()) else {
            return Ok(None);
        };
        if i + 1 == parts.len() {
            return if entry.is_tree() || entry.mode == b"160000" {
                Ok(None)
            } else {
                body_of(reader, entry.oid).map(Some)
            };
        }
        if !entry.is_tree() {
            return Ok(None);
        }
        at = entry.oid;
    }
    Ok(None)
}

/// What a content rule did to `path` in the preview's `commit` (an id of
/// the history before the rewrite): the old file and the new one, diffed.
pub(super) fn blob_diff(root: &Path, run: &RunId, commit: Oid, path: &str) -> Result<BlobChange> {
    let stale = || refuse(RewriteRefusal::Stale, "no such preview: run it again");
    if !store::valid_run(run) {
        return Err(stale());
    }
    let dir = store::run_dir(&super::git_dir(root)?, run);
    let stored = store::read_plan(&dir).ok_or_else(stale)?;
    let recipe = stored.recipe.ok_or_else(stale)?;
    let pack_file = dir.join(store::PACK);
    let pack = PackReader::open(&pack_file, &dir.join(pack::INDEX)).ok_or_else(stale)?;
    // A secret rule's old lines are not shown, whatever the file.
    let redacted = recipe.enabled_rules().any(|(_, r)| {
        matches!(
            &r.kind,
            RuleKind::ContentReplace(ContentReplace {
                find: ContentFind::Secrets { .. },
                ..
            })
        )
    });

    let mut reader = Reader::new(root)?;
    let (_, raw) = reader.get(commit)?;
    let old_tree = Commit::parse(raw)?
        .tree()
        .ok_or_else(|| Error::Backend("a commit without a tree".into()))?;
    let old = file_in(&mut reader, None, old_tree, path)?;

    // The new commit is in the pack; a commit the map does not list keeps
    // its tree, a dropped one has none.
    let map = std::fs::read_to_string(dir.join(store::COMMIT_MAP)).unwrap_or_default();
    let hex = commit.to_hex();
    let successor = map
        .lines()
        .find_map(|l| l.strip_prefix(hex.as_str()))
        .map(|rest| rest.trim().to_string());
    let new = match successor.as_deref().and_then(Oid::from_hex) {
        Some(new_commit) => {
            let body = pack.get(new_commit).ok_or_else(stale)?;
            let tree = Commit::parse(body)?
                .tree()
                .ok_or_else(|| Error::Backend("a commit without a tree".into()))?;
            file_in(&mut reader, Some(&pack), tree, path)?
        }
        None if successor.is_some() => None,
        None => old.clone(),
    };

    let binary = old.iter().chain(new.iter()).any(|b| blobs::is_binary(b))
        || old
            .iter()
            .chain(new.iter())
            .any(|b| std::str::from_utf8(b).is_err());
    let kind = match (&old, &new) {
        (Some(o), Some(n)) if o == n => BlobChangeKind::Unchanged,
        (Some(_), Some(_)) => BlobChangeKind::Modified,
        (Some(_), None) => BlobChangeKind::Removed,
        (None, _) => BlobChangeKind::Unchanged,
    };
    let text = |b: &Option<Vec<u8>>| {
        b.as_deref()
            .and_then(|b| std::str::from_utf8(b).ok())
            .unwrap_or_default()
            .to_string()
    };
    let hunks = if binary || kind == BlobChangeKind::Unchanged {
        Vec::new()
    } else {
        blob_diff_text(&text(&old), &text(&new), redacted)
    };
    Ok(BlobChange {
        run: run.clone(),
        commit,
        path: path.to_string(),
        kind,
        binary,
        redacted,
        old_size: old.as_ref().map_or(0, |b| b.len() as u64),
        new_size: new.as_ref().map_or(0, |b| b.len() as u64),
        hunks,
    })
}
