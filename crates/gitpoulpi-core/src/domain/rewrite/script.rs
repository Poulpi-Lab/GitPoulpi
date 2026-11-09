//! Scripts: the header both the editor and the engine read, the failure
//! text and the result types of `rewrite_script_check` / `rewrite_script_test`
//! (the history-rewrite design, "Scripts").
//!
//! The header is a block of `--!` comment lines at the top of a Lua script:
//!
//! ```lua
//! --! name: Replace an email
//! --! kinds: commit
//! --! window: 0
//! --! needs: []
//! --! param old_email: text "Email to replace"
//! --! param keep: bool "Keep the committer" = false
//! ```
//!
//! Pure: no interpreter here, so the TUI builds its parameter form from the
//! same parser the engine validates with.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use super::plan::FieldChange;
use super::recipe::ScriptLanguage;
use crate::domain::Oid;

/// Per-request timeout of an external script when the settings say nothing.
pub const DEFAULT_SCRIPT_TIMEOUT_MS: u64 = 5_000;
/// Whole-pass timeout of an external script when the settings say nothing.
pub const DEFAULT_SCRIPT_TOTAL_TIMEOUT_S: u64 = 3_600;

/// What external scripts may do on one request. The daemon builds it from the
/// settings (`rewrite.external_scripts`, the timeouts) and the repository's
/// trust store, and replaces whatever a client sent: a frontend cannot grant
/// itself anything. The default is the safe one a bare backend (a test)
/// sees: shell scripts allowed, Python and external Lua off, nothing trusted.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct ScriptPolicy {
    pub shell: bool,
    pub python: bool,
    pub lua: bool,
    /// Per-request timeout in milliseconds.
    pub timeout_ms: u64,
    /// Whole-pass timeout in seconds.
    pub total_timeout_s: u64,
    /// Content hashes ([`script_hash`]) the user trusted, plus the ones the
    /// caller vouched for on this request (a script the user typed).
    pub trusted: Vec<String>,
}

impl Default for ScriptPolicy {
    fn default() -> Self {
        Self {
            shell: true,
            python: false,
            lua: false,
            timeout_ms: DEFAULT_SCRIPT_TIMEOUT_MS,
            total_timeout_s: DEFAULT_SCRIPT_TOTAL_TIMEOUT_S,
            trusted: Vec::new(),
        }
    }
}

impl ScriptPolicy {
    pub fn is_default(&self) -> bool {
        *self == Self::default()
    }

    /// Whether the settings allow `language` (the embedded Lua always).
    pub fn allows(&self, language: ScriptLanguage) -> bool {
        match language {
            ScriptLanguage::Lua => true,
            ScriptLanguage::Shell => self.shell,
            ScriptLanguage::Python => self.python,
            ScriptLanguage::LuaExternal => self.lua,
        }
    }

    pub fn trusts(&self, hash: &str) -> bool {
        self.trusted.iter().any(|t| t == hash)
    }

    /// The policy for a `rewrite.external_scripts` list (`shell`, `python`,
    /// `lua`) and the two timeouts (`0` reads as the default).
    pub fn from_settings(enabled: &[String], timeout_ms: u64, total_timeout_s: u64) -> Self {
        let on = |name: &str| enabled.iter().any(|e| e.trim().eq_ignore_ascii_case(name));
        Self {
            shell: on("shell"),
            python: on("python"),
            lua: on("lua"),
            timeout_ms: if timeout_ms == 0 {
                DEFAULT_SCRIPT_TIMEOUT_MS
            } else {
                timeout_ms
            },
            total_timeout_s: if total_timeout_s == 0 {
                DEFAULT_SCRIPT_TOTAL_TIMEOUT_S
            } else {
                total_timeout_s
            },
            trusted: Vec::new(),
        }
    }
}

/// The SHA-256 of a script's text as lowercase hex: what trust is keyed by,
/// so a changed script asks again.
pub fn script_hash(text: &str) -> String {
    use sha2::{Digest, Sha256};
    use std::fmt::Write;
    let digest = Sha256::digest(text.as_bytes());
    let mut out = String::with_capacity(64);
    for byte in digest {
        let _ = write!(out, "{byte:02x}");
    }
    out
}

/// Whether a script may run, as far as the user's trust goes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ScriptTrust {
    /// The embedded Lua is sandboxed: nothing to trust.
    #[default]
    NotNeeded,
    Trusted,
    /// Nobody trusted this exact text yet.
    Untrusted,
}

/// The largest window a script may ask for.
pub const MAX_WINDOW: u8 = 50;

/// What a script is called on.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ScriptKind {
    Commit,
    Blob,
    Tag,
}

/// The type of a parameter row in the rule form.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ParamKind {
    Text,
    Number,
    Bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ScriptParam {
    pub name: String,
    pub kind: ParamKind,
    pub label: String,
    /// Used when the rule gives no value.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub default: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ScriptHeader {
    pub name: String,
    pub kinds: Vec<ScriptKind>,
    /// Commits before / after the current one given to the script (0..=50).
    pub window: u8,
    /// Extra data the script asks for: `changes`, `path`, `binary`.
    pub needs: Vec<String>,
    pub params: Vec<ScriptParam>,
}

impl Default for ScriptHeader {
    fn default() -> Self {
        Self {
            name: String::new(),
            kinds: vec![ScriptKind::Commit],
            window: 0,
            needs: Vec::new(),
            params: Vec::new(),
        }
    }
}

impl ScriptHeader {
    /// The value of every declared parameter: the rule's, else the default,
    /// else empty (a bool defaults to `false`, a number to `0`).
    pub fn resolve_params(
        &self,
        given: &BTreeMap<String, String>,
    ) -> Result<BTreeMap<String, String>, String> {
        for name in given.keys() {
            if !self.params.iter().any(|p| &p.name == name) {
                return Err(format!("the script declares no parameter `{name}`"));
            }
        }
        let mut out = BTreeMap::new();
        for p in &self.params {
            let value = given
                .get(&p.name)
                .cloned()
                .or_else(|| p.default.clone())
                .unwrap_or_else(|| match p.kind {
                    ParamKind::Text => String::new(),
                    ParamKind::Number => "0".into(),
                    ParamKind::Bool => "false".into(),
                });
            match p.kind {
                ParamKind::Number if value.trim().parse::<f64>().is_err() => {
                    return Err(format!("parameter `{}` is not a number", p.name))
                }
                ParamKind::Bool if value != "true" && value != "false" => {
                    return Err(format!("parameter `{}` is not true or false", p.name))
                }
                _ => {}
            }
            out.insert(p.name.clone(), value);
        }
        Ok(out)
    }

    /// The text parameters that have neither a value nor a default.
    pub fn missing_params(&self, given: &BTreeMap<String, String>) -> Vec<String> {
        self.params
            .iter()
            .filter(|p| p.kind == ParamKind::Text && p.default.is_none())
            .filter(|p| given.get(&p.name).is_none_or(|v| v.is_empty()))
            .map(|p| p.name.clone())
            .collect()
    }
}

const PREFIX: &str = "--!";
const HASH_PREFIX: &str = "#!";

/// Reads the header of a Lua script: the `--!` lines at the top (blank lines
/// and plain comments before them are skipped). A problem names the line.
pub fn parse_header(script: &str) -> Result<ScriptHeader, String> {
    parse_header_for(ScriptLanguage::Lua, script)
}

/// Reads the header of a script in `language`. Lua (embedded or external)
/// uses `--!` lines; shell and Python use `#!` lines (`#! name: ...`), and a
/// shebang (`#!/bin/sh`, `#!/usr/bin/env python3`) or any other `#` comment is
/// skipped like a plain comment.
pub fn parse_header_for(language: ScriptLanguage, script: &str) -> Result<ScriptHeader, String> {
    let hash = matches!(language, ScriptLanguage::Shell | ScriptLanguage::Python);
    let (prefix, plain) = if hash {
        (HASH_PREFIX, "#")
    } else {
        (PREFIX, "--")
    };
    let mut header = ScriptHeader::default();
    for (n, raw) in script.lines().enumerate() {
        let line = raw.trim();
        let is_shebang = hash && line.starts_with("#!") && line[2..].trim_start().starts_with('/');
        let Some(body) = line.strip_prefix(prefix).filter(|_| !is_shebang) else {
            if line.is_empty() || line.starts_with(plain) {
                continue;
            }
            break;
        };
        let at = |msg: String| format!("header line {}: {msg}", n + 1);
        let body = body.trim();
        if let Some(param) = body.strip_prefix("param ") {
            let p = parse_param(param).map_err(at)?;
            if header.params.iter().any(|q| q.name == p.name) {
                return Err(at(format!("parameter `{}` is declared twice", p.name)));
            }
            header.params.push(p);
            continue;
        }
        let Some((key, value)) = body.split_once(':') else {
            return Err(at(format!("expected `key: value`, got `{body}`")));
        };
        let value = value.trim();
        match key.trim() {
            "name" => header.name = value.to_string(),
            "kinds" => {
                header.kinds = list(value)
                    .into_iter()
                    .map(|k| match k.as_str() {
                        "commit" => Ok(ScriptKind::Commit),
                        "blob" => Ok(ScriptKind::Blob),
                        "tag" => Ok(ScriptKind::Tag),
                        other => Err(at(format!("unknown kind `{other}`"))),
                    })
                    .collect::<Result<_, _>>()?;
                if header.kinds.is_empty() {
                    return Err(at("`kinds` is empty".into()));
                }
            }
            "window" => {
                let w: u8 = value
                    .parse()
                    .map_err(|_| at(format!("`window` is not a number: `{value}`")))?;
                if w > MAX_WINDOW {
                    return Err(at(format!("`window` is at most {MAX_WINDOW}")));
                }
                header.window = w;
            }
            "needs" => {
                header.needs = list(value);
                if let Some(bad) = header
                    .needs
                    .iter()
                    .find(|n| !matches!(n.as_str(), "changes" | "path" | "binary" | "blobs"))
                {
                    return Err(at(format!("unknown need `{bad}`")));
                }
            }
            other => return Err(at(format!("unknown header key `{other}`"))),
        }
    }
    Ok(header)
}

/// `a, b` or `[a, b]` or `["a", "b"]`.
fn list(value: &str) -> Vec<String> {
    value
        .trim()
        .trim_start_matches('[')
        .trim_end_matches(']')
        .split(',')
        .map(|s| s.trim().trim_matches('"').trim().to_string())
        .filter(|s| !s.is_empty())
        .collect()
}

/// `name: kind "label" = default`.
fn parse_param(text: &str) -> Result<ScriptParam, String> {
    let (name, rest) = text
        .split_once(':')
        .ok_or_else(|| format!("expected `param name: kind \"label\"`, got `{text}`"))?;
    let name = name.trim();
    if name.is_empty()
        || !name.chars().all(|c| c.is_ascii_alphanumeric() || c == '_')
        || name.starts_with(|c: char| c.is_ascii_digit())
    {
        return Err(format!("`{name}` is not a valid parameter name"));
    }
    let rest = rest.trim();
    let (kind_word, rest) = rest.split_once(char::is_whitespace).unwrap_or((rest, ""));
    let kind = match kind_word {
        "text" => ParamKind::Text,
        "number" => ParamKind::Number,
        "bool" => ParamKind::Bool,
        other => return Err(format!("unknown parameter kind `{other}`")),
    };
    let rest = rest.trim();
    let (label, rest) = match rest.strip_prefix('"') {
        Some(after) => {
            let end = after
                .find('"')
                .ok_or_else(|| "the label has no closing quote".to_string())?;
            (after[..end].to_string(), after[end + 1..].trim())
        }
        None => (name.to_string(), rest),
    };
    let default = match rest {
        "" => None,
        r => Some(
            r.strip_prefix('=')
                .ok_or_else(|| format!("unexpected `{r}` after the label"))?
                .trim()
                .trim_matches('"')
                .to_string(),
        ),
    };
    Ok(ScriptParam {
        name: name.to_string(),
        kind,
        label,
        default,
    })
}

/// What `rewrite_script_check` found: the header, the functions the script
/// defines and everything wrong with it (an empty `problems` is a script the
/// engine accepts). A syntax error is a problem, not an error, so an editor
/// can show it live.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct ScriptCheck {
    pub header: ScriptHeader,
    /// `rewrite_commit`, ... as defined by the script.
    pub functions: Vec<String>,
    pub problems: Vec<String>,
    /// External scripts: the content hash trust is keyed by.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub hash: Option<String>,
    #[serde(default)]
    pub trust: ScriptTrust,
    /// External scripts: the program that will run it (when one was found).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub interpreter: Option<String>,
}

impl ScriptCheck {
    pub fn ok(&self) -> bool {
        self.problems.is_empty()
    }
}

/// One tested commit: what the script would change. Nothing is written.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ScriptTestRow {
    pub commit: Oid,
    pub fields: Vec<FieldChange>,
    /// The script's `log(text)` lines for this commit.
    pub log: Vec<String>,
}

/// The detail of `RewriteRefusal::ScriptFailed`: `<commit hex>: <reason>`,
/// or only the reason when no commit was running (compile errors).
pub fn script_failure(commit: Option<Oid>, reason: &str) -> String {
    match commit {
        Some(c) => format!("{}: {reason}", c.to_hex()),
        None => reason.to_string(),
    }
}

/// Splits [`script_failure`] again.
pub fn parse_script_failure(detail: &str) -> (Option<Oid>, String) {
    if let Some((head, rest)) = detail.split_once(": ") {
        if head.len() == 40 {
            if let Some(oid) = Oid::from_hex(head) {
                return (Some(oid), rest.to_string());
            }
        }
    }
    (None, detail.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE: &str = "\
--! name: Replace an email
--! kinds: commit
--! window: 3
--! needs: [changes]
--! param old_email: text \"Email to replace\"
--! param count: number \"How many\" = 5
--! param dry: bool
-- ordinary comment
function rewrite_commit(c, ctx) end
--! name: ignored after code
";

    #[test]
    fn parses_every_header_line() {
        let h = parse_header(SAMPLE).unwrap();
        assert_eq!(h.name, "Replace an email");
        assert_eq!(h.kinds, vec![ScriptKind::Commit]);
        assert_eq!(h.window, 3);
        assert_eq!(h.needs, vec!["changes"]);
        assert_eq!(h.params.len(), 3);
        assert_eq!(h.params[0].label, "Email to replace");
        assert_eq!(h.params[1].default.as_deref(), Some("5"));
        assert_eq!(h.params[2].label, "dry");
    }

    #[test]
    fn a_script_without_header_gets_defaults() {
        let h = parse_header("function rewrite_commit() end").unwrap();
        assert_eq!(h, ScriptHeader::default());
    }

    #[test]
    fn rejects_bad_headers_with_the_line() {
        for (text, needle) in [
            ("--! window: 99", "header line 1"),
            ("--! window: x", "not a number"),
            ("--! kinds: movie", "unknown kind"),
            ("--! bogus: 1", "unknown header key"),
            ("--! param 1x: text", "valid parameter name"),
            ("--! param a: blob", "unknown parameter kind"),
            ("--! param a: text\n--! param a: text", "declared twice"),
            ("--! needs: [sudo]", "unknown need"),
        ] {
            let e = parse_header(text).unwrap_err();
            assert!(e.contains(needle), "{text}: {e}");
        }
    }

    #[test]
    fn resolves_params_with_defaults_and_types() {
        let h = parse_header(SAMPLE).unwrap();
        let mut given = BTreeMap::new();
        given.insert("old_email".to_string(), "a@b".to_string());
        let got = h.resolve_params(&given).unwrap();
        assert_eq!(got["count"], "5");
        assert_eq!(got["dry"], "false");
        given.insert("count".into(), "many".into());
        assert!(h.resolve_params(&given).unwrap_err().contains("number"));
        given.insert("count".into(), "1".into());
        given.insert("nope".into(), "1".into());
        assert!(h.resolve_params(&given).unwrap_err().contains("nope"));
        assert_eq!(h.missing_params(&BTreeMap::new()), vec!["old_email"]);
    }

    #[test]
    fn failure_text_round_trips() {
        let oid = Oid([7; 20]);
        let text = script_failure(Some(oid), "too long");
        assert_eq!(parse_script_failure(&text), (Some(oid), "too long".into()));
        assert_eq!(
            parse_script_failure("syntax error: x"),
            (None, "syntax error: x".into())
        );
    }
}

#[cfg(test)]
mod external_tests {
    use super::*;

    #[test]
    fn hash_headers_skip_the_shebang_and_read_hash_bang_lines() {
        let text = "#!/bin/sh\n#! name: Rename\n# a plain comment\n#! window: 2\n#! param who: text \"Who\"\necho hi\n#! name: ignored after code\n";
        for language in [ScriptLanguage::Shell, ScriptLanguage::Python] {
            let h = parse_header_for(language, text).unwrap();
            assert_eq!(h.name, "Rename", "{language:?}");
            assert_eq!(h.window, 2);
            assert_eq!(h.params.len(), 1);
        }
        // `--!` is the Lua prefix, and a `#!` line is not a header there.
        assert!(parse_header_for(ScriptLanguage::Lua, text)
            .unwrap()
            .name
            .is_empty());
        let lua = "--! name: Lua one\nreturn\n";
        assert_eq!(
            parse_header_for(ScriptLanguage::LuaExternal, lua)
                .unwrap()
                .name,
            "Lua one"
        );
        // A shebang with a space before the path is still a shebang.
        let spaced = "#! /usr/bin/env python3\n#! name: X\n";
        assert_eq!(
            parse_header_for(ScriptLanguage::Python, spaced)
                .unwrap()
                .name,
            "X"
        );
        // A bad header line in a hash script names the line.
        let err = parse_header_for(ScriptLanguage::Shell, "#!/bin/sh\n#! nonsense\n").unwrap_err();
        assert!(err.contains("header line 2"), "{err}");
    }

    #[test]
    fn the_hash_is_of_the_exact_text() {
        assert_eq!(
            script_hash(""),
            "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
        );
        assert_ne!(script_hash("a\n"), script_hash("a"));
        assert_eq!(script_hash("x"), script_hash("x"));
    }

    #[test]
    fn the_policy_follows_the_settings_and_trusts_by_hash() {
        let p = ScriptPolicy::from_settings(&["shell".into(), "Python".into()], 0, 0);
        assert!(p.shell && p.python && !p.lua);
        assert_eq!(p.timeout_ms, DEFAULT_SCRIPT_TIMEOUT_MS);
        assert_eq!(p.total_timeout_s, DEFAULT_SCRIPT_TOTAL_TIMEOUT_S);
        assert!(
            p.allows(ScriptLanguage::Lua),
            "the embedded Lua is always on"
        );
        assert!(p.allows(ScriptLanguage::Python) && !p.allows(ScriptLanguage::LuaExternal));
        let off = ScriptPolicy::from_settings(&[], 10, 20);
        assert!(!off.shell && off.timeout_ms == 10 && off.total_timeout_s == 20);
        let mut p = ScriptPolicy::default();
        assert!(p.shell && !p.python && !p.lua && p.is_default());
        p.trusted.push(script_hash("a"));
        assert!(p.trusts(&script_hash("a")) && !p.trusts(&script_hash("b")));
        assert!(!p.is_default());
    }

    #[test]
    fn languages_have_names_settings_and_extensions() {
        for language in ScriptLanguage::ALL {
            assert_eq!(ScriptLanguage::from_name(language.name()), Some(language));
        }
        assert!(!ScriptLanguage::Lua.is_external());
        assert!(ScriptLanguage::Shell.is_external());
        assert_eq!(ScriptLanguage::Lua.setting_name(), None);
        assert_eq!(ScriptLanguage::LuaExternal.setting_name(), Some("lua"));
        assert_eq!(
            ScriptLanguage::from_extension("a/b.PS1"),
            Some(ScriptLanguage::Shell)
        );
        assert_eq!(
            ScriptLanguage::from_extension("x.py"),
            Some(ScriptLanguage::Python)
        );
        assert_eq!(ScriptLanguage::from_extension("x"), None);
        // The wire names.
        assert_eq!(
            serde_json::to_string(&ScriptLanguage::LuaExternal).unwrap(),
            "\"lua_external\""
        );
    }

    #[test]
    fn a_recipe_saved_forgets_its_policy() {
        use super::super::{Recipe, Scope};
        let mut r = Recipe::new("x", Scope::default(), Vec::new());
        r.script_policy.python = true;
        r.script_policy.trusted.push("aa".into());
        assert!(r.saved().script_policy.is_default());
        let json = serde_json::to_string(&r.saved()).unwrap();
        assert!(!json.contains("script_policy"), "{json}");
    }
}
