//! History rewrite, slice S10 (the history-rewrite design,
//! "Scripts"): the embedded Lua script rule, `rewrite_script_check` and
//! `rewrite_script_test`. Expected values come from plain `git` and from the
//! scripts' own arithmetic, never from the engine under test. These checks
//! need the `rewrite-scripts` feature of the backend under test.

use std::collections::BTreeMap;

use gitpoulpi_core::domain::rewrite::script::parse_script_failure;
use gitpoulpi_core::domain::rewrite::{
    Recipe, RewriteField, RewriteRefusal, Rule, RuleKind, Scope, ScriptLanguage, ScriptPolicy,
    ScriptRef, ScriptSource, SignaturePolicy,
};
use gitpoulpi_core::{Error, GitBackend};

use super::rewrite::{
    alice, bob, chain, git_text, history, object_files, preview, run, run_dirs, BASE,
};
use crate::FixtureRepo;

fn script(source: &str, params: &[(&str, &str)]) -> ScriptRef {
    ScriptRef {
        language: ScriptLanguage::Lua,
        source: ScriptSource::Inline {
            text: source.to_string(),
        },
        params: params
            .iter()
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect::<BTreeMap<_, _>>(),
    }
}

fn recipe(script: ScriptRef) -> Recipe {
    let mut r = Recipe::new(
        "script",
        Scope::default(),
        vec![Rule::new(RuleKind::Script(script))],
    );
    r.options.signatures = Some(SignaturePolicy::Strip);
    r
}

/// Seven commits (the fixture's own, then six alternating Alice and Bob).
fn start() -> (FixtureRepo, Vec<String>) {
    let authors: Vec<(String, &str)> = (0..6)
        .map(|i| (if i % 2 == 0 { alice(i) } else { bob(i) }, ""))
        .collect();
    let (repo, _) = chain(&authors);
    let ids = history(&repo);
    (repo, ids)
}

fn lines(repo: &FixtureRepo, format: &str) -> Vec<String> {
    git_text(
        repo,
        &[
            "log",
            "--reverse",
            "--topo-order",
            &format!("--format={format}"),
            "HEAD",
        ],
    )
    .lines()
    .map(str::to_string)
    .collect()
}

fn snapshot(repo: &FixtureRepo) -> (String, Vec<String>, Vec<String>) {
    (
        git_text(repo, &["for-each-ref"]),
        object_files(repo),
        run_dirs(repo),
    )
}

/// The failure of a preview: the typed refusal's commit and reason.
fn script_failure(
    backend: &dyn GitBackend,
    repo: &FixtureRepo,
    source: &str,
) -> (Option<gitpoulpi_core::domain::Oid>, String) {
    let before = snapshot(repo);
    let result =
        backend.rewrite_preview(&repo.repo_ref(), &recipe(script(source, &[])), &mut |_| {
            std::ops::ControlFlow::Continue(())
        });
    let out = match result {
        Err(Error::RewriteRefused(RewriteRefusal::ScriptFailed, detail)) => {
            parse_script_failure(&detail)
        }
        other => panic!("expected ScriptFailed for `{source}`, got {other:?}"),
    };
    assert_eq!(snapshot(repo), before, "a failed script changes nothing");
    out
}

/// The header, the functions and the problems come back; a syntax error is
/// a problem and not an error; other languages are unsupported.
pub fn check_rewrite_script_check_reads_the_header_and_reports_problems(backend: &dyn GitBackend) {
    let (repo, _) = start();
    let r = repo.repo_ref();
    let text = "--! name: Replace an email\n--! window: 2\n--! param old: text \"Old email\"\n--! param n: number \"How many\" = 3\nfunction rewrite_commit(c, ctx) end\n";
    let check = backend
        .rewrite_script_check(&r, &script(text, &[]), &ScriptPolicy::default())
        .expect("check");
    assert!(check.ok(), "{:?}", check.problems);
    assert_eq!(check.header.name, "Replace an email");
    assert_eq!(check.header.window, 2);
    assert_eq!(check.header.params.len(), 2);
    assert_eq!(check.header.params[1].default.as_deref(), Some("3"));
    assert_eq!(check.functions, vec!["rewrite_commit"]);

    let broken = backend
        .rewrite_script_check(
            &r,
            &script("function rewrite_commit(\n", &[]),
            &ScriptPolicy::default(),
        )
        .expect("a syntax error is a problem, not an error");
    assert!(!broken.ok());
    assert!(
        broken.problems[0].contains("script:"),
        "{:?}",
        broken.problems
    );

    let none = backend
        .rewrite_script_check(&r, &script("local x = 1", &[]), &ScriptPolicy::default())
        .unwrap();
    assert!(none.problems.iter().any(|p| p.contains("rewrite_commit")));

    let later = backend
        .rewrite_script_check(
            &r,
            &script("--! needs: [changes]\nfunction rewrite_commit() end", &[]),
            &ScriptPolicy::default(),
        )
        .unwrap();
    assert!(later.problems.iter().any(|p| p.contains("not supported")));

    let bad_param = backend
        .rewrite_script_check(
            &r,
            &script(text, &[("n", "many"), ("old", "a")]),
            &ScriptPolicy::default(),
        )
        .unwrap();
    assert!(bad_param.problems.iter().any(|p| p.contains("number")));
}

/// A script maps identities, dates and messages with typed parameters; the
/// result is what plain git reads back, and commits it leaves alone keep
/// their ids.
pub fn check_rewrite_script_rule_changes_people_dates_and_messages(backend: &dyn GitBackend) {
    let (repo, before_ids) = start();
    let source = r#"--! param old_email: text "Email to replace"
--! param new_email: text "New email"
--! param shift: number "Seconds" = 60
--! param loud: bool "Shout" = false
function rewrite_commit(c, ctx)
  local p = ctx.params
  if c.author.email == p.old_email then
    local msg = c.message
    if p.loud then msg = string.upper(msg) end
    return {
      author = { email = p.new_email, time = c.author.time + p.shift },
      message = msg,
    }
  end
end
"#;
    let params = [
        ("old_email", "alice@example.com"),
        ("new_email", "carol@example.com"),
        ("loud", "true"),
    ];
    let (plan, outcome) = run(backend, &repo, &recipe(script(source, &params)));
    // Alice wrote three; every commit after the first of them gets a new
    // parent, so six are rewritten.
    assert_eq!(plan.counts.commits_changed, 6);
    assert_eq!(outcome.commits_changed, 6);

    let got = lines(&repo, "%ae|%at|%s");
    let after_ids = history(&repo);
    for (i, line) in got.iter().enumerate().skip(1) {
        let i = i as i64 - 1; // the chain's own index
        if i % 2 == 0 {
            assert_eq!(
                *line,
                format!("carol@example.com|{}|COMMIT {i}", BASE + i * 3600 + 60)
            );
            assert_ne!(after_ids[i as usize + 1], before_ids[i as usize + 1]);
        } else {
            assert_eq!(
                *line,
                format!("bob@example.com|{}|commit {i}", BASE + i * 3600)
            );
        }
    }
    // The fixture's first commit is not Alice's: same object, same id.
    assert_eq!(after_ids[0], before_ids[0]);

    // An unknown parameter is a refusal, not silently ignored.
    let bad = backend.rewrite_preview(
        &repo.repo_ref(),
        &recipe(script(source, &[("nope", "1")])),
        &mut |_| std::ops::ControlFlow::Continue(()),
    );
    assert!(matches!(
        bad,
        Err(Error::RewriteRefused(RewriteRefusal::ScriptFailed, _))
    ));
}

/// `ctx.index`, `ctx.total` and `ctx.state` follow the processing order:
/// parents first, state replaced by what the script returns.
pub fn check_rewrite_script_carries_state_between_commits(backend: &dyn GitBackend) {
    let (repo, _) = start();
    let source = r#"function rewrite_commit(c, ctx)
  local count = (ctx.state.count or 0) + 1
  local seen = ctx.state.seen or ''
  return {
    message = 'n' .. count .. ' of ' .. ctx.total .. ' index ' .. ctx.index .. ' seen ' .. seen .. '\n',
    state = { count = count, seen = seen .. count },
  }
end
"#;
    run(backend, &repo, &recipe(script(source, &[])));
    let got = lines(&repo, "%s");
    let mut seen = String::new();
    for (i, line) in got.iter().enumerate() {
        assert_eq!(
            *line,
            format!("n{} of 7 index {i} seen {seen}", i + 1).trim_end(),
            "{i}"
        );
        seen.push_str(&(i + 1).to_string());
    }
}

/// The window: `before` holds the parents as already rewritten, `after`
/// the next commits as they were.
pub fn check_rewrite_script_window_is_rewritten_before_and_original_after(
    backend: &dyn GitBackend,
) {
    let (repo, _) = start();
    let original = lines(&repo, "%s");
    let source = r#"--! window: 2
function rewrite_commit(c, ctx)
  local b1, b2 = ctx.window.before[1], ctx.window.before[2]
  local a1, a2 = ctx.window.after[1], ctx.window.after[2]
  return { message =
    (b1 and b1.subject or '^') .. '/' .. (b2 and b2.subject or '^') ..
    ' > ' .. ctx.index .. ' < ' ..
    (a1 and a1.subject or '$') .. '/' .. (a2 and a2.subject or '$') .. '\n' }
end
"#;
    let ids = history(&repo);
    run(backend, &repo, &recipe(script(source, &[])));
    let mut rewritten: Vec<String> = Vec::new();
    for i in 0..original.len() {
        let b1 = i.checked_sub(1).map(|j| rewritten[j].clone());
        let b2 = i.checked_sub(2).map(|j| rewritten[j].clone());
        let want = format!(
            "{}/{} > {i} < {}/{}",
            b1.as_deref().unwrap_or("^"),
            b2.as_deref().unwrap_or("^"),
            original.get(i + 1).map_or("$", String::as_str),
            original.get(i + 2).map_or("$", String::as_str),
        );
        rewritten.push(want);
    }
    assert_eq!(lines(&repo, "%s"), rewritten);
    assert_eq!(history(&repo).len(), ids.len());
}

/// A failing script is a typed `ScriptFailed` naming the commit and the
/// reason, and changes nothing.
pub fn check_rewrite_script_failures_are_typed_with_commit_and_reason(backend: &dyn GitBackend) {
    let (repo, ids) = start();
    let (commit, reason) = script_failure(
        backend,
        &repo,
        "function rewrite_commit(c, ctx) if ctx.index == 2 then error('boom at ' .. c.message) end end",
    );
    assert_eq!(commit.map(|c| c.to_hex()), Some(ids[2].clone()));
    assert!(reason.contains("boom at commit 1"), "{reason}");

    let (commit, reason) = script_failure(
        backend,
        &repo,
        "function rewrite_commit(c) return { tree = 'x' } end",
    );
    assert_eq!(commit.map(|c| c.to_hex()), Some(ids[0].clone()));
    assert!(reason.contains("`tree` belongs to the engine"), "{reason}");

    let (_, reason) = script_failure(
        backend,
        &repo,
        "function rewrite_commit(c) return { colour = 'red' } end",
    );
    assert!(reason.contains("unknown field `colour`"), "{reason}");

    let (_, reason) = script_failure(
        backend,
        &repo,
        "function rewrite_commit(c) return { author = { email = 3 } } end",
    );
    assert!(reason.contains("author.email"), "{reason}");

    // No commit is named for a script that does not compile.
    let (commit, reason) = script_failure(backend, &repo, "function rewrite_commit(\n");
    assert_eq!(commit, None);
    assert!(reason.contains("script:"), "{reason}");
}

/// Instruction, memory and recursion limits turn a runaway script into a
/// typed failure.
pub fn check_rewrite_script_limits_stop_runaway_scripts(backend: &dyn GitBackend) {
    let (repo, ids) = start();
    let (commit, reason) = script_failure(
        backend,
        &repo,
        "function rewrite_commit(c) while true do end end",
    );
    assert_eq!(commit.map(|c| c.to_hex()), Some(ids[0].clone()));
    assert!(reason.contains("too long"), "{reason}");

    let (_, reason) = script_failure(
        backend,
        &repo,
        "function rewrite_commit(c) local s = string.rep('x', 1 << 28) return { message = s } end",
    );
    assert!(reason.to_lowercase().contains("memory"), "{reason}");

    let (_, reason) = script_failure(
        backend,
        &repo,
        "function f(n) return 1 + f(n + 1) end function rewrite_commit(c) return f(1) end",
    );
    assert!(reason.contains("stack overflow"), "{reason}");

    // The top level of the script is limited too.
    let (_, reason) = script_failure(backend, &repo, "while true do end");
    assert!(reason.contains("too long"), "{reason}");
}

/// No file, clock, process or module access: each is a failure the script
/// cannot get around.
pub fn check_rewrite_script_has_no_file_clock_or_module_access(backend: &dyn GitBackend) {
    let (repo, ids) = start();
    for code in [
        "io.open('/etc/hostname')",
        "os.time()",
        "os.clock()",
        "os.execute('true')",
        "require('os')",
        "dofile('/etc/passwd')",
        "loadfile('/etc/passwd')",
        "load('return 1')()",
        "debug.getinfo(1)",
        "math.random()",
        "pcall(error)",
    ] {
        let (commit, reason) = script_failure(
            backend,
            &repo,
            &format!("function rewrite_commit(c) {code} end"),
        );
        assert_eq!(commit.map(|c| c.to_hex()), Some(ids[0].clone()), "{code}");
        assert!(
            reason.contains("nil") || reason.contains("attempt to"),
            "{code}: {reason}"
        );
    }
}

/// Two runs of the same script on the same history give the same objects,
/// whatever the table order or the hash seed of the interpreter.
pub fn check_rewrite_script_output_is_deterministic(backend: &dyn GitBackend) {
    let (repo, _) = start();
    let source = r#"function rewrite_commit(c, ctx)
  local t = { zeta = 1, alpha = 2, mid = 3, [10] = 'x', [2] = 'y' }
  local names = {}
  for k, v in pairs(t) do names[#names + 1] = tostring(k) .. v end
  return {
    author = {
      name = table.concat(names, '-') .. stable_hash(c.id, 'salt'),
      time = c.author.time + math.floor(rand(ctx.index) * 1000),
    },
    message = regex_replace(c.message, '[0-9]+', '#') .. tostring({}),
  }
end
"#;
    let r = recipe(script(source, &[]));
    let first = preview(backend, &repo, &r);
    let second = preview(backend, &repo, &r);
    assert_eq!(first.digest, second.digest);
    assert_eq!(first.counts, second.counts);
    let news = |p: &gitpoulpi_core::domain::rewrite::RewritePlan| -> Vec<_> {
        p.rows.iter().map(|r| (r.old, r.new)).collect()
    };
    assert_eq!(news(&first), news(&second));
    assert!(!news(&first).is_empty());
    // Sorted key order, not hash order.
    let name = first.rows[0]
        .fields
        .iter()
        .find(|f| f.field == RewriteField::AuthorName)
        .map(|f| f.new.clone())
        .expect("the name changed");
    assert!(name.starts_with("2y-10x-alpha2-mid3-zeta1"), "{name}");
    backend
        .rewrite_discard(&repo.repo_ref(), &first.run)
        .unwrap();
    backend
        .rewrite_discard(&repo.repo_ref(), &second.run)
        .unwrap();
}

/// `rewrite_script_test` runs the script on a selection, carries the state
/// across it, returns the changes and the log, and writes nothing.
pub fn check_rewrite_script_test_reports_changes_without_writing(backend: &dyn GitBackend) {
    let (repo, ids) = start();
    let source = r#"function rewrite_commit(c, ctx)
  log('seen ' .. ctx.index .. ' of ' .. ctx.total)
  local count = (ctx.state.count or 0) + 1
  return { message = 'n' .. count .. '\n', state = { count = count } }
end
"#;
    let before = snapshot(&repo);
    let pick = [
        gitpoulpi_core::domain::Oid::from_hex(&ids[4]).unwrap(),
        gitpoulpi_core::domain::Oid::from_hex(&ids[2]).unwrap(),
    ];
    let rows = backend
        .rewrite_script_test(
            &repo.repo_ref(),
            &script(source, &[]),
            &ScriptPolicy::default(),
            &pick,
        )
        .expect("test");
    assert_eq!(snapshot(&repo), before, "a test writes nothing");
    // History order, whatever the order asked for.
    assert_eq!(
        rows.iter().map(|r| r.commit.to_hex()).collect::<Vec<_>>(),
        vec![ids[2].clone(), ids[4].clone()]
    );
    let message = |i: usize| {
        rows[i]
            .fields
            .iter()
            .find(|f| f.field == RewriteField::Message)
            .map(|f| f.new.clone())
    };
    assert_eq!(message(0).as_deref(), Some("n1\n"));
    assert_eq!(message(1).as_deref(), Some("n2\n"));
    assert_eq!(rows[0].log, vec!["seen 0 of 2"]);
    assert_eq!(rows[1].log, vec!["seen 1 of 2"]);

    // A failing script names the commit it failed on.
    let failing = backend.rewrite_script_test(
        &repo.repo_ref(),
        &script("function rewrite_commit() error('no') end", &[]),
        &ScriptPolicy::default(),
        &pick,
    );
    match failing {
        Err(Error::RewriteRefused(RewriteRefusal::ScriptFailed, detail)) => {
            let (commit, reason) = parse_script_failure(&detail);
            assert_eq!(commit.map(|c| c.to_hex()), Some(ids[2].clone()));
            assert!(reason.contains("no"), "{reason}");
        }
        other => panic!("expected ScriptFailed, got {other:?}"),
    }
    assert_eq!(snapshot(&repo), before);

    assert!(backend
        .rewrite_script_test(
            &repo.repo_ref(),
            &script(source, &[]),
            &ScriptPolicy::default(),
            &[],
        )
        .unwrap()
        .is_empty());
}

/// A script in a repository file, a path that leaves the repository, and a
/// language this build does not run.
pub fn check_rewrite_script_file_source_and_unsupported_languages(backend: &dyn GitBackend) {
    let (repo, _) = start();
    let dir = repo.root().join(".gitpoulpi").join("rewrite");
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(
        dir.join("shout.lua"),
        "function rewrite_commit(c) return { message = string.upper(c.message) } end\n",
    )
    .unwrap();
    let mut file = script("", &[]);
    file.source = ScriptSource::File {
        path: ".gitpoulpi/rewrite/shout.lua".into(),
    };
    run(backend, &repo, &recipe(file));
    assert_eq!(lines(&repo, "%s")[1], "COMMIT 0");

    for path in ["../outside.lua", "/etc/passwd"] {
        let mut escaping = script("", &[]);
        escaping.source = ScriptSource::File { path: path.into() };
        let result = backend.rewrite_preview(&repo.repo_ref(), &recipe(escaping), &mut |_| {
            std::ops::ControlFlow::Continue(())
        });
        assert!(
            matches!(
                result,
                Err(Error::RewriteRefused(RewriteRefusal::InvalidRule, _))
            ),
            "{path}: {result:?}"
        );
    }

    // The external languages are refused before anything runs: Python and
    // external Lua are off by default, a shell script nobody trusted does not
    // run (the external contract checks cover the rest).
    for (language, refusal) in [
        (ScriptLanguage::Python, RewriteRefusal::ScriptsDisabled),
        (ScriptLanguage::LuaExternal, RewriteRefusal::ScriptsDisabled),
        (ScriptLanguage::Shell, RewriteRefusal::ScriptNotTrusted),
    ] {
        // A POSIX shebang pins the shell dialect: without one Windows picks
        // PowerShell, and a machine without it would answer `ToolMissing`
        // before the trust check.
        let source = if language == ScriptLanguage::Shell {
            "#!/bin/sh\necho"
        } else {
            "echo"
        };
        let mut other = script(source, &[]);
        other.language = language;
        let result = backend.rewrite_preview(&repo.repo_ref(), &recipe(other), &mut |_| {
            std::ops::ControlFlow::Continue(())
        });
        assert!(
            matches!(&result, Err(Error::RewriteRefused(r, _)) if *r == refusal),
            "{language:?}: {result:?}"
        );
    }
}
