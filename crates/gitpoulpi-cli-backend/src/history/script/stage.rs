//! Loading a script, checking it, running it over the commits of a pass and
//! testing it on a few commits. The embedded Lua and the external languages
//! (S11, `external.rs`) share everything but the call itself.

use std::collections::{HashMap, HashSet};
use std::path::{Component, Path};

use gitpoulpi_core::domain::rewrite::script::{
    parse_header_for, script_failure, ParamKind, ScriptCheck, ScriptHeader, ScriptKind,
    ScriptTestRow,
};
use gitpoulpi_core::domain::rewrite::{
    script_hash, GitDate, RewriteRefusal, ScriptLanguage, ScriptPolicy, ScriptRef, ScriptSource,
    ScriptTrust, Tool,
};
use gitpoulpi_core::domain::Oid;
use gitpoulpi_core::{Error, Result};
use serde_json::{json, Value};

use super::contract::{parse_response, CommitView, Patch, PersonPatch, Request, Summary};
use super::external::{ExternalProc, ExternalSpec, Fail};
use super::lua_rt::{Limits, Runtime};
use crate::history::engine::field_changes;
use crate::history::objects::{Commit, Person};
use crate::history::rules::Work;
use crate::history::walk::{self, refuse, Reader};
use crate::tools::ToolProbe;

const MAX_SCRIPT_BYTES: u64 = 1 << 20;

/// What a script may do here: the settings' and trust store's policy, and the
/// probe that finds interpreters.
#[derive(Clone, Copy)]
pub(crate) struct ScriptEnv<'a> {
    pub policy: &'a ScriptPolicy,
    pub probe: &'a ToolProbe,
}

/// How a loaded script runs.
enum How {
    Lua(Runtime),
    External(ExternalSpec),
}

/// A script that passed the checks, with its parameters resolved.
pub(crate) struct Script {
    header: ScriptHeader,
    how: How,
    params: Value,
}

fn failed(commit: Option<Oid>, reason: &str) -> Error {
    Error::RewriteRefused(RewriteRefusal::ScriptFailed, script_failure(commit, reason))
}

pub(crate) fn supported(_: &ScriptRef) -> Result<()> {
    Ok(())
}

/// The script text. A file is read from the repository: a relative path
/// that stays inside it.
fn source_of(root: &Path, script: &ScriptRef) -> Result<String> {
    match &script.source {
        ScriptSource::Inline { text } => Ok(text.clone()),
        ScriptSource::File { path } => {
            let rel = Path::new(path);
            let escapes = rel.is_absolute()
                || rel
                    .components()
                    .any(|c| !matches!(c, Component::Normal(_) | Component::CurDir));
            if escapes {
                return Err(refuse(
                    RewriteRefusal::InvalidRule,
                    &format!("the script path `{path}` must stay inside the repository"),
                ));
            }
            let full = root.join(rel);
            let meta = std::fs::metadata(&full).map_err(|e| {
                refuse(
                    RewriteRefusal::InvalidRule,
                    &format!("cannot read the script `{path}`: {e}"),
                )
            })?;
            if meta.len() > MAX_SCRIPT_BYTES {
                return Err(refuse(
                    RewriteRefusal::InvalidRule,
                    &format!("the script `{path}` is over {MAX_SCRIPT_BYTES} bytes"),
                ));
            }
            std::fs::read_to_string(&full).map_err(|e| {
                refuse(
                    RewriteRefusal::InvalidRule,
                    &format!("cannot read the script `{path}`: {e}"),
                )
            })
        }
    }
}

/// What this version of the engine cannot run, as editor problems. An
/// external script has no function list: `functions` is `None`.
fn unsupported(header: &ScriptHeader, functions: Option<&[String]>) -> Vec<String> {
    let mut out = Vec::new();
    if !header.kinds.contains(&ScriptKind::Commit) {
        out.push("`kinds` must include `commit`: only commit scripts run".to_string());
    }
    if header.kinds.iter().any(|k| *k != ScriptKind::Commit) {
        out.push("blob and tag scripts are not supported yet".to_string());
    }
    if !header.needs.is_empty() {
        out.push(format!(
            "`needs: {}` is not supported yet",
            header.needs.join(", ")
        ));
    }
    let Some(functions) = functions else {
        return out;
    };
    for f in functions {
        if f != "rewrite_commit" {
            out.push(format!("`{f}` is not supported yet"));
        }
    }
    if !functions.iter().any(|f| f == "rewrite_commit") {
        out.push("the script defines no `rewrite_commit(c, ctx)`".to_string());
    }
    out
}

// ------------------------------------------------------------ interpreters

/// Which shell dialect a shell script is written in: a `.ps1` file or a
/// PowerShell shebang is PowerShell, a `.sh` / `.bash` file or a POSIX
/// shebang is POSIX; otherwise the platform's: PowerShell on Windows, POSIX
/// elsewhere.
fn powershell_dialect(path: Option<&str>, text: &str) -> bool {
    let ext = path
        .and_then(|p| p.rsplit_once('.'))
        .map(|(_, e)| e.to_ascii_lowercase());
    match ext.as_deref() {
        Some("ps1") => return true,
        Some("sh" | "bash") => return false,
        _ => {}
    }
    let first = text.lines().next().unwrap_or("");
    // A shebang names an interpreter: `#!/bin/sh`, `#!/usr/bin/env pwsh`,
    // `#!pwsh`. `#! name: ...` is a header line of the script's own (see the
    // header parser) and says nothing about the dialect.
    let is_header = first
        .strip_prefix("#!")
        .and_then(|rest| rest.split_whitespace().next())
        .is_some_and(|word| word.ends_with(':'));
    if first.starts_with("#!") && !is_header {
        return first.contains("pwsh") || first.contains("powershell");
    }
    cfg!(windows)
}

/// The program, its leading arguments and the file name for `script`.
fn external_command(
    root: &Path,
    script: &ScriptRef,
    text: &str,
    env: ScriptEnv<'_>,
) -> Result<(std::path::PathBuf, Vec<String>, String)> {
    let missing = |what: &str| {
        refuse(
            RewriteRefusal::ToolMissing,
            &format!("{what} is not installed, so this script cannot run"),
        )
    };
    let strings = |args: &[&str]| args.iter().map(|a| (*a).to_string()).collect::<Vec<_>>();
    match script.language {
        ScriptLanguage::Lua => unreachable!("the embedded Lua is not an external process"),
        ScriptLanguage::Python => {
            let program = env
                .probe
                .program(Tool::Python, root)
                .ok_or_else(|| missing("Python"))?;
            let launcher = program
                .file_stem()
                .is_some_and(|s| s.eq_ignore_ascii_case("py"));
            let lead = if launcher {
                strings(&["-3", "-I", "-u"])
            } else {
                strings(&["-I", "-u"])
            };
            Ok((program, lead, "script.py".into()))
        }
        ScriptLanguage::LuaExternal => {
            let program = env
                .probe
                .program(Tool::Lua, root)
                .ok_or_else(|| missing("Lua"))?;
            Ok((program, Vec::new(), "script.lua".into()))
        }
        ScriptLanguage::Shell => {
            let path = match &script.source {
                ScriptSource::File { path } => Some(path.as_str()),
                ScriptSource::Inline { .. } => None,
            };
            if powershell_dialect(path, text) {
                let program = env
                    .probe
                    .program(Tool::Pwsh, root)
                    .ok_or_else(|| missing("PowerShell (pwsh)"))?;
                let lead = strings(&[
                    "-NoProfile",
                    "-NonInteractive",
                    "-ExecutionPolicy",
                    "Bypass",
                    "-File",
                ]);
                return Ok((program, lead, "script.ps1".into()));
            }
            if let Some(program) = env.probe.program(Tool::Bash, root) {
                return Ok((
                    program,
                    strings(&["--noprofile", "--norc"]),
                    "script.sh".into(),
                ));
            }
            let program = env
                .probe
                .program(Tool::Sh, root)
                .ok_or_else(|| missing("a POSIX shell (bash or sh)"))?;
            Ok((program, Vec::new(), "script.sh".into()))
        }
    }
}

fn params_json(
    header: &ScriptHeader,
    resolved: &std::collections::BTreeMap<String, String>,
) -> Value {
    let mut m = serde_json::Map::new();
    for p in &header.params {
        let raw = &resolved[&p.name];
        let v = match p.kind {
            ParamKind::Text => json!(raw),
            ParamKind::Bool => json!(raw == "true"),
            ParamKind::Number => {
                let f: f64 = raw.trim().parse().unwrap_or(0.0);
                if f.fract() == 0.0 && f.abs() < 9.0e15 {
                    json!(f as i64)
                } else {
                    json!(f)
                }
            }
        };
        m.insert(p.name.clone(), v);
    }
    Value::Object(m)
}

/// Compiles and inspects a script. The `How` is `Some` when the script
/// loads, the check says what else is wrong with it.
///
/// An external script is refused here, before anything runs, when its
/// language is switched off (`ScriptsDisabled`) or its interpreter is
/// missing (`ToolMissing`); whether the user trusts it is in the check
/// (`trust`) and `load` refuses an untrusted one.
fn analyse(
    root: &Path,
    script: &ScriptRef,
    limits: Limits,
    env: ScriptEnv<'_>,
) -> Result<(Option<(How, Value)>, ScriptCheck)> {
    supported(script)?;
    if script.language.is_external() && !env.policy.allows(script.language) {
        return Err(refuse(
            RewriteRefusal::ScriptsDisabled,
            &format!(
                "{} scripts are switched off: add \"{}\" to `rewrite.external_scripts` in the settings",
                script.language.name(),
                script.language.setting_name().unwrap_or("")
            ),
        ));
    }
    let source = source_of(root, script)?;
    let mut check = ScriptCheck::default();
    match parse_header_for(script.language, &source) {
        Ok(h) => check.header = h,
        Err(e) => {
            check.problems.push(e);
            return Ok((None, check));
        }
    }
    let params = match check.header.resolve_params(&script.params) {
        Ok(p) => params_json(&check.header, &p),
        Err(e) => {
            check.problems.push(e);
            json!({})
        }
    };
    if script.language.is_external() {
        let (program, lead_args, file_name) = external_command(root, script, &source, env)?;
        let hash = script_hash(&source);
        check.trust = if env.policy.trusts(&hash) {
            ScriptTrust::Trusted
        } else {
            ScriptTrust::Untrusted
        };
        check.hash = Some(hash);
        check.interpreter = Some(program.display().to_string());
        check.problems.extend(unsupported(&check.header, None));
        let spec = ExternalSpec {
            program,
            lead_args,
            file_name,
            text: source,
            timeout: std::time::Duration::from_millis(env.policy.timeout_ms),
            total: std::time::Duration::from_secs(env.policy.total_timeout_s),
        };
        return Ok((Some((How::External(spec), params)), check));
    }
    let runtime = match Runtime::new(&source, limits) {
        Ok(rt) => rt,
        Err(e) => {
            check.problems.push(e);
            return Ok((None, check));
        }
    };
    check.functions = runtime.functions();
    let problems = unsupported(&check.header, Some(&check.functions));
    check.problems.extend(problems);
    Ok((Some((How::Lua(runtime), params)), check))
}

/// A request for a made-up commit, marked `"dry_run": true`: what the
/// handshake of a check sends, so a script that answers it has the contract
/// right without touching real history.
fn dry_request(params: &Value) -> Value {
    let person = Person::parse(b"Dry Run <dry-run@example.invalid> 0 +0000");
    let commit = CommitView {
        id: Oid([0; 20]),
        parents: Vec::new(),
        tree: None,
        author: person.clone(),
        committer: person,
        message: "dry run\n".into(),
        signed: false,
    };
    let state = json!({});
    let request = Request {
        index: 0,
        total: 1,
        commit: &commit,
        before: &[],
        after: &[],
        params,
        state: &state,
    };
    let mut value = request.json();
    value["dry_run"] = json!(true);
    value
}

/// The handshake: a process of the script answers the dry request with
/// something the contract accepts.
fn handshake(spec: &ExternalSpec, params: &Value) -> std::result::Result<(), String> {
    let mut proc = ExternalProc::start(spec)?;
    let result = proc.call(&dry_request(params), &mut || Ok(()));
    proc.end();
    match result {
        Ok((answer, _)) => answer_patch(&answer).map(|_| ()),
        Err(Fail::Script(why)) => Err(why),
        Err(Fail::Engine(e)) => Err(e.to_string()),
    }
}

/// An external answer: `{"error": "..."}` is the script giving up.
fn answer_patch(answer: &Value) -> std::result::Result<Patch, String> {
    if let Some(message) = answer.get("error").and_then(Value::as_str) {
        return Err(format!("the script reported an error: {message}"));
    }
    parse_response(answer)
}

pub(crate) fn check(root: &Path, script: &ScriptRef, env: ScriptEnv<'_>) -> Result<ScriptCheck> {
    let (loaded, mut check) = analyse(root, script, Limits::default(), env)?;
    if let Some((How::External(spec), params)) = &loaded {
        // An untrusted script is not run, not even for a handshake.
        if check.trust == ScriptTrust::Trusted && check.problems.is_empty() {
            if let Err(why) = handshake(spec, params) {
                check
                    .problems
                    .push(format!("the script failed its handshake: {why}"));
            }
        }
    }
    Ok(check)
}

/// A script ready to run; a script with any problem is `ScriptFailed`, an
/// external one the user has not trusted `ScriptNotTrusted` (the detail is its
/// content hash).
pub(crate) fn load(root: &Path, script: &ScriptRef, env: ScriptEnv<'_>) -> Result<Script> {
    load_with(root, script, Limits::default(), env)
}

pub(super) fn load_with(
    root: &Path,
    script: &ScriptRef,
    limits: Limits,
    env: ScriptEnv<'_>,
) -> Result<Script> {
    let (loaded, check) = analyse(root, script, limits, env)?;
    if check.trust == ScriptTrust::Untrusted {
        return Err(refuse(
            RewriteRefusal::ScriptNotTrusted,
            check.hash.as_deref().unwrap_or(""),
        ));
    }
    match (loaded, check.problems.first()) {
        (Some((how, params)), None) => Ok(Script {
            header: check.header,
            how,
            params,
        }),
        (_, problem) => Err(failed(None, problem.map_or("the script cannot run", |p| p))),
    }
}

fn summary(w: &Work, id: Oid) -> Summary {
    let text = String::from_utf8_lossy(w.message());
    Summary {
        id,
        author: w.author.clone(),
        committer: w.committer.clone(),
        subject: text.lines().next().unwrap_or("").to_string(),
    }
}

fn patch_person(p: &mut Person, patch: &PersonPatch) {
    if let Some(n) = &patch.name {
        p.name.clone_from(n);
    }
    if let Some(e) = &patch.email {
        p.email.clone_from(e);
    }
    if patch.time.is_some() || patch.offset_minutes.is_some() {
        p.set_date(GitDate::new(
            patch.time.unwrap_or(p.seconds),
            patch.offset_minutes.unwrap_or(p.offset),
        ));
    }
}

/// Applies an answer; whether the commit changed.
fn apply(w: &mut Work, patch: &Patch) -> std::result::Result<bool, String> {
    let before = (w.author.clone(), w.committer.clone(), w.message().to_vec());
    if let Some(p) = &patch.author {
        let person = w
            .author
            .as_mut()
            .ok_or("the commit has no readable author to change")?;
        patch_person(person, p);
    }
    if let Some(p) = &patch.committer {
        let person = w
            .committer
            .as_mut()
            .ok_or("the commit has no readable committer to change")?;
        patch_person(person, p);
    }
    if let Some(m) = &patch.message {
        if std::str::from_utf8(w.message()).is_err() {
            return Err("the message is not UTF-8 text, a script cannot replace it".into());
        }
        if m.as_bytes() != w.message() {
            w.message = Some(m.clone().into_bytes());
        }
    }
    Ok(before != (w.author.clone(), w.committer.clone(), w.message().to_vec()))
}

/// Runs `rewrite_commit` on every active commit in order. `order` is the
/// ids of `works`, parents first. Returns the number of commits changed.
/// `log` receives each commit's `log(...)` lines (index in `works`).
pub(crate) fn run_stage(
    script: &Script,
    works: &mut [Work],
    order: &[Oid],
    active: &[bool],
    tick: &mut dyn FnMut(u64) -> Result<()>,
    log: &mut dyn FnMut(usize, Vec<String>),
) -> Result<u64> {
    let at: HashMap<Oid, usize> = order.iter().enumerate().map(|(i, o)| (*o, i)).collect();
    let mut first_parent: Vec<Option<usize>> = Vec::with_capacity(works.len());
    let mut parents: Vec<Vec<Oid>> = Vec::with_capacity(works.len());
    for w in works.iter() {
        let p = w.commit.parents()?;
        first_parent.push(p.first().and_then(|o| at.get(o).copied()));
        parents.push(p);
    }
    let mut first_child: Vec<Option<usize>> = vec![None; works.len()];
    for (k, p) in first_parent.iter().enumerate() {
        if let Some(p) = p {
            first_child[*p].get_or_insert(k);
        }
    }
    let window = usize::from(script.header.window);
    let total = active.iter().filter(|a| **a).count() as u64;
    // One process per pass for an external script; killed with its tree
    // when this function ends, however it ends.
    let mut external = match &script.how {
        How::External(spec) => Some(ExternalProc::start(spec).map_err(|e| failed(None, &e))?),
        How::Lua(_) => None,
    };
    let mut last_tick = std::time::Instant::now();
    let mut state = json!({});
    let mut index = 0u64;
    let mut changed = 0u64;
    for k in 0..works.len() {
        if !active[k] {
            continue;
        }
        if index % 64 == 0 {
            tick(index)?;
        }
        let mut before = Vec::new();
        let mut cur = first_parent[k];
        while let (Some(p), true) = (cur, before.len() < window) {
            before.push(summary(&works[p], order[p]));
            cur = first_parent[p];
        }
        let mut after = Vec::new();
        let mut cur = first_child[k];
        while let (Some(c), true) = (cur, after.len() < window) {
            // The originals: what the rewrite has not touched yet.
            after.push(original_summary(&works[c], order[c]));
            cur = first_child[c];
        }
        let w = &works[k];
        let view = CommitView {
            id: order[k],
            parents: parents[k].clone(),
            tree: w.commit.tree(),
            author: w.author.clone(),
            committer: w.committer.clone(),
            message: String::from_utf8_lossy(w.message()).into_owned(),
            signed: w.commit.is_signed(),
        };
        let request = Request {
            index,
            total,
            commit: &view,
            before: &before,
            after: &after,
            params: &script.params,
            state: &state,
        };
        let (answer, lines) = match (&script.how, external.as_mut()) {
            (How::Lua(runtime), _) => runtime
                .call("rewrite_commit", &request.lua_args())
                .map_err(|e| failed(Some(order[k]), &e))?,
            (How::External(_), Some(proc)) => proc
                .call(&request.json(), &mut || {
                    // Progress and cancel at most every 100 ms while a
                    // script thinks (the Lua path ticks every 64 commits).
                    if last_tick.elapsed() >= std::time::Duration::from_millis(100) {
                        last_tick = std::time::Instant::now();
                        tick(index)?;
                    }
                    Ok(())
                })
                .map_err(|fail| match fail {
                    Fail::Script(why) => failed(Some(order[k]), &why),
                    Fail::Engine(e) => e,
                })?,
            (How::External(_), None) => unreachable!("started above"),
        };
        let patch = answer_patch(&answer).map_err(|e| failed(Some(order[k]), &e))?;
        if let Some(s) = &patch.state {
            state = s.clone();
        }
        let mut lines = lines;
        lines.extend(patch.log.clone());
        if !lines.is_empty() {
            log(k, lines);
        }
        if apply(&mut works[k], &patch).map_err(|e| failed(Some(order[k]), &e))? {
            changed += 1;
        }
        index += 1;
    }
    if let Some(proc) = external.as_mut() {
        proc.end();
    }
    Ok(changed)
}

fn original_summary(w: &Work, id: Oid) -> Summary {
    let text = String::from_utf8_lossy(w.commit.message());
    Summary {
        id,
        author: w.author0.clone(),
        committer: w.committer0.clone(),
        subject: text.lines().next().unwrap_or("").to_string(),
    }
}

/// Runs the script on `commits` as a run over a selection would (the same
/// history order, the state carried from one to the next) and reports what
/// it changed, writing nothing. Ancestors and the commits after HEAD are
/// read for the window, so the cost grows with the history like a preview's.
pub(crate) fn test(
    root: &Path,
    script: &ScriptRef,
    env: ScriptEnv<'_>,
    commits: &[Oid],
) -> Result<Vec<ScriptTestRow>> {
    let script = load(root, script, env)?;
    if commits.is_empty() {
        return Ok(Vec::new());
    }
    let wanted: HashSet<Oid> = commits.iter().copied().collect();
    let mut tips: Vec<String> = commits.iter().map(|o| o.to_hex()).collect();
    tips.push("HEAD".into());
    let order = match walk::rev_list(root, &tips, &[]) {
        Ok(order) => order,
        // No HEAD (an unborn branch): the selection alone.
        Err(_) => walk::rev_list(root, &tips[..tips.len() - 1], &[])?,
    };
    if let Some(missing) = commits.iter().find(|c| !order.contains(c)) {
        return Err(refuse(
            RewriteRefusal::InvalidRule,
            &format!("{} is not a commit", missing.to_hex()),
        ));
    }
    let mut reader = Reader::new(root)?;
    let raw = reader.commits(&order, |_| Ok(()))?;
    let mut works = Vec::with_capacity(raw.len());
    for bytes in raw {
        works.push(Work::new(Commit::parse(bytes)?));
    }
    let active: Vec<bool> = order.iter().map(|o| wanted.contains(o)).collect();
    let mut logs: HashMap<usize, Vec<String>> = HashMap::new();
    run_stage(
        &script,
        &mut works,
        &order,
        &active,
        &mut |_| Ok(()),
        &mut |k, lines| {
            logs.insert(k, lines);
        },
    )?;
    Ok(order
        .iter()
        .enumerate()
        .filter(|(k, _)| active[*k])
        .map(|(k, id)| ScriptTestRow {
            commit: *id,
            fields: field_changes(&works[k], &works[k].committer, false, false),
            log: logs.remove(&k).unwrap_or_default(),
        })
        .collect())
}

#[cfg(test)]
mod tests {
    use super::*;
    use gitpoulpi_core::domain::rewrite::ToolStatus;

    fn external(language: ScriptLanguage, text: &str) -> ScriptRef {
        ScriptRef {
            language,
            source: ScriptSource::Inline { text: text.into() },
            params: Default::default(),
        }
    }

    fn refusal(result: Result<ScriptCheck>) -> (RewriteRefusal, String) {
        match result {
            Err(Error::RewriteRefused(r, d)) => (r, d),
            other => panic!("expected a refusal, got {:?}", other.map(|c| c.problems)),
        }
    }

    #[test]
    fn a_missing_interpreter_is_refused_as_tool_missing_naming_it() {
        let probe = ToolProbe::forced(Tool::Python, ToolStatus::Missing);
        let policy = ScriptPolicy {
            python: true,
            ..ScriptPolicy::default()
        };
        let env = ScriptEnv {
            policy: &policy,
            probe: &probe,
        };
        let dir = std::env::temp_dir();
        let (reason, detail) = refusal(check(&dir, &external(ScriptLanguage::Python, "x"), env));
        assert_eq!(reason, RewriteRefusal::ToolMissing);
        assert!(detail.contains("Python"), "{detail}");
    }

    #[test]
    fn the_settings_are_asked_before_the_interpreter() {
        // Off in the policy and missing on the machine: the setting wins, so
        // the user is told to switch it on rather than to install anything.
        let probe = ToolProbe::forced(Tool::Python, ToolStatus::Missing);
        let policy = ScriptPolicy::default();
        let env = ScriptEnv {
            policy: &policy,
            probe: &probe,
        };
        let (reason, _) = refusal(check(
            &std::env::temp_dir(),
            &external(ScriptLanguage::Python, "x"),
            env,
        ));
        assert_eq!(reason, RewriteRefusal::ScriptsDisabled);
    }

    #[test]
    fn the_shell_dialect_follows_the_file_name_then_the_shebang_then_the_platform() {
        assert!(powershell_dialect(Some("a/b.ps1"), "echo"));
        assert!(!powershell_dialect(Some("a/b.sh"), "#!/usr/bin/env pwsh\n"));
        assert!(!powershell_dialect(Some("b.BASH"), ""));
        assert!(powershell_dialect(None, "#!/usr/bin/env pwsh\n"));
        assert!(powershell_dialect(None, "#!/usr/bin/powershell\n"));
        assert!(!powershell_dialect(None, "#!/bin/sh\n"));
        assert_eq!(powershell_dialect(None, "echo"), cfg!(windows));
    }

    #[test]
    fn a_header_line_is_not_a_shebang() {
        // `#! name: ...` is the script's metadata, not an interpreter line:
        // the platform's dialect still applies (PowerShell on Windows).
        let header = "#! name: Uppercase author names\n#! kinds: commit\n";
        assert_eq!(powershell_dialect(None, header), cfg!(windows));
        assert!(powershell_dialect(None, "#!pwsh\n"));
    }
}
