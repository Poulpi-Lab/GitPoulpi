use std::collections::HashMap;

use super::*;

struct Rng(u64);
impl Rng {
    fn next(&mut self) -> u64 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        self.0
    }
    fn below(&mut self, n: usize) -> usize {
        (self.next() % n as u64) as usize
    }
}

fn oid(rng: &mut Rng) -> Oid {
    let mut hex = String::new();
    for _ in 0..3 {
        hex.push_str(&format!("{:016x}", rng.next()));
    }
    hex.truncate(40);
    Oid::from_hex(&hex).unwrap()
}

fn node(id: Oid, parents: Vec<Oid>, ts: i64, refs: Vec<RefKind>) -> CommitNode {
    CommitNode {
        id,
        parents,
        summary: format!("subject of {}", &id.to_hex()[..4]),
        message: String::new(),
        author_name: "Ada".into(),
        author_email: "ada@example.org".into(),
        timestamp: ts,
        refs,
    }
}

fn local(name: &str) -> RefKind {
    RefKind::LocalBranch { name: name.into() }
}

/// What a replayed script built: id -> parent ids, in emission order.
struct Replay {
    parents: HashMap<String, Vec<String>>,
}

fn word_or_quoted(rest: &str) -> (String, &str) {
    let rest = rest.trim_start();
    if let Some(r) = rest.strip_prefix('"') {
        let end = r.find('"').expect("unterminated quote");
        (r[..end].to_string(), &r[end + 1..])
    } else {
        let end = rest.find(' ').unwrap_or(rest.len());
        (rest[..end].to_string(), &rest[end..])
    }
}

fn quoted_attr(rest: &str, key: &str) -> Option<String> {
    let at = rest.find(&format!("{key}: \""))?;
    let (v, _) = word_or_quoted(&rest[at + key.len() + 2..]);
    Some(v)
}

/// Replays a script the way Mermaid does and panics on anything Mermaid
/// rejects: unknown branch, duplicate branch or id, merging a branch into
/// itself or with equal heads, merging an empty branch.
fn replay(script: &str) -> Replay {
    let mut main = "main".to_string();
    let mut lines = script.lines().filter(|l| !l.trim().is_empty()).peekable();
    let mut saw_header = false;
    if let Some(l) = lines.peek() {
        if let Some(at) = l.find("'mainBranchName': '") {
            let r = &l[at + "'mainBranchName': '".len()..];
            main = r[..r.find('\'').unwrap()].to_string();
            lines.next();
        }
    }
    let mut branches: HashMap<String, Option<String>> = HashMap::new();
    branches.insert(main.clone(), None);
    let mut cur = main;
    let mut parents: HashMap<String, Vec<String>> = HashMap::new();
    // The branch each commit was created on (Mermaid's `commit.branch`).
    let mut created_on: HashMap<String, String> = HashMap::new();
    for line in lines {
        if line.starts_with("gitGraph ") {
            assert!(!saw_header, "two headers");
            saw_header = true;
            continue;
        }
        assert!(saw_header, "statement before the header: {line}");
        let line = line.trim();
        let (verb, rest) = line.split_once(' ').unwrap_or((line, ""));
        match verb {
            "branch" => {
                let (name, _) = word_or_quoted(rest);
                assert!(!branches.contains_key(&name), "duplicate branch {name}");
                let head = branches[&cur].clone();
                branches.insert(name.clone(), head);
                cur = name;
            }
            "checkout" => {
                let (name, _) = word_or_quoted(rest);
                assert!(branches.contains_key(&name), "checkout of unknown {name}");
                cur = name;
            }
            "commit" => {
                let id = quoted_attr(rest, "id").expect("commit without id");
                assert!(!parents.contains_key(&id), "duplicate id {id}");
                let head = branches[&cur].clone();
                parents.insert(id.clone(), head.into_iter().collect());
                created_on.insert(id.clone(), cur.clone());
                branches.insert(cur.clone(), Some(id));
            }
            "merge" => {
                let (other, tail) = word_or_quoted(rest);
                let id = quoted_attr(tail, "id").expect("merge without id");
                assert!(!parents.contains_key(&id), "duplicate id {id}");
                assert_ne!(other, cur, "merge of a branch into itself");
                let mine = branches[&cur].clone().expect("merge into an empty branch");
                assert_ne!(created_on[&mine], other, "head was created on {other}");
                let theirs = branches
                    .get(&other)
                    .unwrap_or_else(|| panic!("merge of unknown {other}"))
                    .clone()
                    .expect("merge of an empty branch");
                assert_ne!(mine, theirs, "merge of equal heads");
                parents.insert(id.clone(), vec![mine, theirs]);
                created_on.insert(id.clone(), cur.clone());
                branches.insert(cur.clone(), Some(id));
            }
            other => panic!("unknown statement {other}: {line}"),
        }
    }
    assert!(saw_header || script.is_empty());
    Replay { parents }
}

/// The replayed parents of `id`, looking through octopus intermediates.
fn real_parents(rp: &Replay, id: &str) -> Vec<String> {
    let mut out = Vec::new();
    for p in &rp.parents[id] {
        if p.contains(" (octopus ") {
            // An intermediate merge stands for its own parents, in order.
            let inner = real_parents(rp, p);
            for q in inner {
                if !out.contains(&q) {
                    out.push(q);
                }
            }
        } else if !out.contains(p) {
            out.push(p.clone());
        }
    }
    out
}

/// Checks the replay of `script` is exactly the DAG of `commits`.
fn assert_same_dag(commits: &[CommitNode], script: &str) {
    let rp = replay(script);
    let selected: HashMap<String, &CommitNode> =
        commits.iter().map(|c| (c.id.to_hex(), c)).collect();
    let by_prefix = |short: &str| -> String {
        let hits: Vec<&String> = selected.keys().filter(|k| k.starts_with(short)).collect();
        assert_eq!(hits.len(), 1, "id {short} not unique or unknown");
        hits[0].clone()
    };
    let mut seen = 0;
    for id in rp.parents.keys().filter(|i| !i.contains(" (octopus ")) {
        seen += 1;
        let full = by_prefix(id.split(' ').next().unwrap());
        let want: Vec<String> = selected[&full]
            .parents
            .iter()
            .map(|p| p.to_hex())
            .filter(|p| selected.contains_key(p))
            .collect();
        let got: Vec<String> = real_parents(&rp, id)
            .iter()
            .map(|p| by_prefix(p.split(' ').next().unwrap()))
            .collect();
        let mut want_set = want.clone();
        want_set.sort();
        want_set.dedup();
        let mut got_set = got.clone();
        got_set.sort();
        assert_eq!(got_set, want_set, "parents of {id} in\n{script}");
        if want.len() >= 2 {
            assert_eq!(
                got.first(),
                want.first(),
                "first parent of {id} in\n{script}"
            );
        }
    }
    assert_eq!(seen, selected.len(), "every commit once in\n{script}");
}

fn opts(detail: MermaidDetail) -> MermaidOptions {
    MermaidOptions {
        detail,
        ..MermaidOptions::default()
    }
}

#[test]
fn a_linear_run_is_commits_on_main() {
    let mut r = Rng(7);
    let (a, b) = (oid(&mut r), oid(&mut r));
    let nodes = vec![
        node(b, vec![a], 2, vec![local("main")]),
        node(a, vec![], 1, vec![]),
    ];
    let s = mermaid_git_graph(&nodes, Some(b), &opts(MermaidDetail::Refs));
    let lines: Vec<&str> = s.lines().collect();
    assert_eq!(lines[0], "gitGraph LR:");
    assert!(lines[1].starts_with("  commit id: \""));
    assert!(lines[2].ends_with("type: HIGHLIGHT"), "{s}");
    assert_same_dag(&nodes, &s);
}

#[test]
fn a_merge_uses_the_branch_names_and_tags() {
    let mut r = Rng(11);
    let (a, b, c, m) = (oid(&mut r), oid(&mut r), oid(&mut r), oid(&mut r));
    let nodes = vec![
        node(
            m,
            vec![b, c],
            5,
            vec![local("main"), RefKind::Tag { name: "v1".into() }],
        ),
        node(c, vec![a], 4, vec![local("feature/x")]),
        node(b, vec![a], 3, vec![]),
        node(a, vec![], 1, vec![]),
    ];
    let s = mermaid_git_graph(&nodes, Some(m), &opts(MermaidDetail::Refs));
    assert!(s.contains("branch \"feature/x\"\n"), "{s}");
    assert!(s.contains("merge \"feature/x\" id: "), "{s}");
    assert!(s.contains("tag: \"v1\""), "{s}");
    assert_same_dag(&nodes, &s);
    let flat = mermaid_git_graph(&nodes, Some(m), &opts(MermaidDetail::Structure));
    assert!(!flat.contains("tag:") && !flat.contains("HIGHLIGHT") && !flat.contains("feature"));
    assert!(flat.contains("branch b1"), "{flat}");
}

#[test]
fn subjects_and_full_levels_extend_the_id() {
    let mut r = Rng(3);
    let a = oid(&mut r);
    let mut n = node(a, vec![], 86_400 * 365, vec![]);
    n.summary = "say \"hi\" and a very long subject that goes on and on forever".into();
    let s = mermaid_git_graph(&[n.clone()], None, &opts(MermaidDetail::Subjects));
    assert!(s.contains("say 'hi'"), "quotes become apostrophes: {s}");
    assert!(s.contains("..."), "{s}");
    let f = mermaid_git_graph(&[n], None, &opts(MermaidDetail::Full));
    assert!(f.contains("- Ada, 1971-01-01"), "{f}");
}

#[test]
fn orientation_fence_and_main_name() {
    let mut r = Rng(5);
    let a = oid(&mut r);
    let nodes = vec![node(a, vec![], 1, vec![local("trunk")])];
    let o = MermaidOptions {
        orientation: MermaidOrientation::Tb,
        fence: true,
        ..MermaidOptions::default()
    };
    let s = mermaid_git_graph(&nodes, None, &o);
    assert!(
        s.starts_with(
            "```mermaid\n%%{init: {'gitGraph': {'mainBranchName': 'trunk'}}}%%\ngitGraph TB:\n"
        ),
        "{s}"
    );
    assert!(s.ends_with("```\n"));
}

#[test]
fn a_selection_gap_gives_disconnected_lanes() {
    let mut r = Rng(9);
    let (a, b, c) = (oid(&mut r), oid(&mut r), oid(&mut r));
    // c's parent b is outside the selection.
    let nodes = vec![node(c, vec![b], 3, vec![]), node(a, vec![], 1, vec![])];
    let s = mermaid_git_graph(&nodes, None, &opts(MermaidDetail::Structure));
    assert_same_dag(&nodes, &s);
    assert!(s.lines().nth(1).unwrap().starts_with("  branch "), "{s}");
}

#[test]
fn an_octopus_becomes_sequential_merges() {
    let mut r = Rng(13);
    let (a, b, c, d, m) = (
        oid(&mut r),
        oid(&mut r),
        oid(&mut r),
        oid(&mut r),
        oid(&mut r),
    );
    let nodes = vec![
        node(m, vec![b, c, d], 6, vec![]),
        node(d, vec![a], 5, vec![]),
        node(c, vec![a], 4, vec![]),
        node(b, vec![a], 3, vec![]),
        node(a, vec![], 1, vec![]),
    ];
    let s = mermaid_git_graph(&nodes, None, &opts(MermaidDetail::Refs));
    assert!(s.contains("(octopus 1/2)"), "{s}");
    assert_same_dag(&nodes, &s);
}

#[test]
fn a_merged_branch_that_moved_on_is_merged_through_a_pointer() {
    let mut r = Rng(17);
    let (a, f1, f2, x, m) = (
        oid(&mut r),
        oid(&mut r),
        oid(&mut r),
        oid(&mut r),
        oid(&mut r),
    );
    // feature has f1, f2; main merges only f1.
    let nodes = vec![
        node(m, vec![x, f1], 7, vec![]),
        node(f2, vec![f1], 6, vec![local("feature")]),
        node(x, vec![a], 5, vec![local("main")]),
        node(f1, vec![a], 3, vec![]),
        node(a, vec![], 1, vec![]),
    ];
    let s = mermaid_git_graph(&nodes, None, &opts(MermaidDetail::Refs));
    assert_same_dag(&nodes, &s);
}

#[test]
fn random_dags_always_replay_validly() {
    let mut rng = Rng(0x9E37_79B9_7F4A_7C15);
    let names = [
        "main",
        "dev",
        "feature/x-1",
        "merge",
        "a\"b",
        "fix it",
        "release",
    ];
    for _ in 0..400 {
        let n = 1 + rng.below(24);
        let mut ids = Vec::new();
        let mut all = Vec::new();
        for i in 0..n {
            let id = oid(&mut rng);
            let mut ps = Vec::new();
            if i > 0 {
                for _ in 0..rng.below(4) {
                    let p = ids[rng.below(i)];
                    if !ps.contains(&p) {
                        ps.push(p);
                    }
                }
                if ps.is_empty() && rng.below(6) != 0 {
                    ps.push(ids[i - 1]);
                }
            }
            let mut refs = Vec::new();
            if rng.below(3) == 0 {
                refs.push(local(names[rng.below(names.len())]));
            }
            if rng.below(5) == 0 {
                refs.push(RefKind::Tag {
                    name: format!("v{i} \"x\""),
                });
            }
            if rng.below(6) == 0 {
                refs.push(RefKind::RemoteBranch {
                    remote: "origin".into(),
                    name: names[rng.below(names.len())].into(),
                });
            }
            all.push(node(
                id,
                ps,
                (i as i64) * 10 + rng.below(25) as i64 - 12,
                refs,
            ));
            ids.push(id);
        }
        // A random subset, in a random-ish (newest first) order.
        let mut sel: Vec<CommitNode> = all.iter().filter(|_| rng.below(4) != 0).cloned().collect();
        sel.reverse();
        let head = sel.first().map(|c| c.id);
        for detail in MermaidDetail::ALL {
            for orientation in MermaidOrientation::ALL {
                let o = MermaidOptions {
                    detail,
                    orientation,
                    fence: false,
                    subject_width: 12,
                };
                let s = mermaid_git_graph(&sel, head, &o);
                assert_same_dag(&sel, &s);
            }
        }
    }
}

#[test]
fn dump_for_the_real_parser() {
    // `GP_MERMAID_DUMP=<dir>` writes random scripts to feed mermaid's own
    // parser (`node check.mjs`); a no-op otherwise.
    let Ok(dir) = std::env::var("GP_MERMAID_DUMP") else {
        return;
    };
    let mut rng = Rng(0xABCD_EF01_2345);
    let names = ["main", "dev", "feature/x-1", "merge", "release", "a b"];
    let mut chunks = Vec::new();
    for _ in 0..150 {
        let n = 1 + rng.below(18);
        let mut ids = Vec::new();
        let mut all = Vec::new();
        for i in 0..n {
            let id = oid(&mut rng);
            let mut ps = Vec::new();
            if i > 0 {
                for _ in 0..rng.below(4) {
                    let p = ids[rng.below(i)];
                    if !ps.contains(&p) {
                        ps.push(p);
                    }
                }
            }
            let mut refs = Vec::new();
            if rng.below(3) == 0 {
                refs.push(local(names[rng.below(names.len())]));
            }
            if rng.below(5) == 0 {
                refs.push(RefKind::Tag {
                    name: format!("v{i}"),
                });
            }
            all.push(node(id, ps, i as i64 * 10, refs));
            ids.push(id);
        }
        all.reverse();
        let head = all.first().map(|c| c.id);
        for detail in MermaidDetail::ALL {
            let o = MermaidOptions {
                detail,
                orientation: MermaidOrientation::Tb,
                fence: false,
                subject_width: 20,
            };
            chunks.push(mermaid_git_graph(&all, head, &o));
        }
    }
    std::fs::write(
        format!("{dir}/scripts.txt"),
        chunks.join("\n=====\n").trim_end().to_string() + "\n",
    )
    .unwrap();
}

#[test]
fn from_here_to_tip_keeps_the_commits_between_and_drops_the_rest() {
    let mut r = Rng(21);
    let (a, b, c, side, tip) = (
        oid(&mut r),
        oid(&mut r),
        oid(&mut r),
        oid(&mut r),
        oid(&mut r),
    );
    // a <- b <- c <- tip; `side` is a child of a that tip does not contain.
    let nodes = vec![
        node(side, vec![a], 5, vec![]),
        node(tip, vec![c], 4, vec![]),
        node(c, vec![b], 3, vec![]),
        node(b, vec![a], 2, vec![]),
        node(a, vec![], 1, vec![]),
    ];
    let ids = |v: Vec<CommitNode>| v.into_iter().map(|n| n.id).collect::<Vec<_>>();
    assert_eq!(ids(commits_to_tip(&nodes, &[b], tip)), vec![tip, c, b]);
    assert_eq!(ids(commits_to_tip(&nodes, &[a], tip)), vec![tip, c, b, a]);
    assert!(commits_to_tip(&nodes, &[side], tip).is_empty());
}
