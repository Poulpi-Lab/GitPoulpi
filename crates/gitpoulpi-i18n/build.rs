//! Compiles `locales/*.toml` into `$OUT_DIR/messages.rs`: static tables and
//! typed accessors. Every check of the i18n design §5.2 that
//! can run at build time runs here and fails the build with a message naming
//! the file and key. Runs offline; only `toml` is needed.
//!
//! Message shapes (see `docs/developer/i18n.md`):
//!   key = "text {arg} {$term}"
//!   key = { git = "...", local = "..." }          per-vocabulary override
//!   key = { one = "...", other = "..." }          plural (each value may be a
//!                                                 `{ git, local }` table too)
//! Term tokens `{$name}` / `{$Name}` / `{$NAME}` expand from `[term]` into
//! four static variants (host pull/merge x git vocabulary english/local).

use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::fmt::Write as _;
use std::path::{Path, PathBuf};
use std::{env, fs};

use toml::{Table, Value};

/// (file stem, default endonym used when the file is absent). The order is
/// the `Lang` discriminant order in `src/lib.rs`; `qps` comes last.
const LANGS: &[(&str, &str)] = &[
    ("en", "English"),
    ("fr", "Français"),
    ("es", "Español"),
    ("pt-BR", "Português (Brasil)"),
    ("pt-PT", "Português (Portugal)"),
    ("it", "Italiano"),
    ("de", "Deutsch"),
    ("zh-CN", "简体中文"),
    ("ja", "日本語"),
    ("ru", "Русский"),
    ("ko", "한국어"),
    ("nl", "Nederlands"),
    ("pl", "Polski"),
    ("uk", "Українська"),
    ("cs", "Čeština"),
    ("sv", "Svenska"),
    ("tr", "Türkçe"),
];
const QPS: usize = LANGS.len();
const N_LANGS: usize = LANGS.len() + 1;
/// The order `SHIPPED` lists languages in (the Settings picker order).
const SHIPPED_ORDER: &[&str] = &[
    "en", "fr", "es", "pt-BR", "pt-PT", "it", "de", "zh-CN", "ja", "ko", "ru", "uk", "pl", "cs",
    "nl", "sv", "tr",
];

/// The cargo feature env var of language `i`: `CARGO_FEATURE_LANG_PT_BR`.
/// English is always built; the pseudo-locale is `lang-qps`.
fn is_built(i: usize) -> bool {
    if i == 0 {
        return true;
    }
    let stem = if i == QPS { "qps" } else { LANGS[i].0 };
    let var = format!(
        "CARGO_FEATURE_LANG_{}",
        stem.to_uppercase().replace('-', "_")
    );
    env::var_os(var).is_some()
}
const N_VARIANTS: usize = 8;
const KEYWORDS: &[&str] = &[
    "as", "break", "const", "continue", "crate", "else", "enum", "extern", "false", "fn", "for",
    "if", "impl", "in", "let", "loop", "match", "mod", "move", "mut", "pub", "ref", "return",
    "self", "Self", "static", "struct", "super", "trait", "true", "type", "unsafe", "use", "where",
    "while", "async", "await", "dyn", "abstract", "become", "box", "do", "final", "macro",
    "override", "priv", "typeof", "unsized", "virtual", "yield", "try", "gen",
];

#[derive(Clone, Copy, PartialEq)]
enum Case {
    As,
    Cap,
    Upper,
    /// `{$^pr}`: every word capitalized ("Pull Request"), for English titles.
    Title,
}

#[derive(Clone)]
enum Tok {
    Lit(String),
    Arg(String),
    Term(String, Case),
}

#[derive(Clone)]
struct Form {
    git: Vec<Tok>,
    local: Vec<Tok>,
}

struct Msg {
    plural: bool,
    /// `[one, other]` when plural, else one entry.
    forms: Vec<Form>,
    /// CLDR `few` / `many` (ru, uk, pl, cs); fall back to `other`.
    few: Option<Form>,
    many: Option<Form>,
}

impl Msg {
    /// Every form with its CLDR name, `one` first.
    fn all_forms(&self) -> Vec<(&'static str, &Form)> {
        let mut v: Vec<(&'static str, &Form)> = Vec::new();
        for (i, f) in self.forms.iter().enumerate() {
            v.push((
                if self.plural && i == 1 {
                    "other"
                } else {
                    "one"
                },
                f,
            ));
        }
        if let Some(f) = &self.few {
            v.push(("few", f));
        }
        if let Some(f) = &self.many {
            v.push(("many", f));
        }
        v
    }
}

enum TermVal {
    One(String),
    Host { pull: String, merge: String },
}

struct Term {
    git: TermVal,
    local: TermVal,
}

struct Catalog {
    name: String,
    msgs: BTreeMap<String, Msg>,
    terms: BTreeMap<String, Term>,
    /// ja, ko, zh-CN: the "English git words" option acts here.
    english_option: bool,
}

#[derive(Clone, PartialEq, Eq, Hash)]
enum Seg {
    Lit(String),
    Arg(usize),
}

fn is_ident(s: &str) -> bool {
    let mut c = s.chars();
    matches!(c.next(), Some('a'..='z'))
        && c.all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_')
        && !KEYWORDS.contains(&s)
}

fn tokenize(s: &str) -> Result<Vec<Tok>, String> {
    let mut out = Vec::new();
    let mut lit = String::new();
    let mut it = s.chars().peekable();
    while let Some(c) = it.next() {
        match c {
            '{' if it.peek() == Some(&'{') => {
                it.next();
                lit.push('{');
            }
            '}' if it.peek() == Some(&'}') => {
                it.next();
                lit.push('}');
            }
            '}' => return Err("unbalanced `}` (write `}}` for a literal brace)".into()),
            '{' => {
                let mut name = String::new();
                loop {
                    match it.next() {
                        Some('}') => break,
                        Some('{') | None => {
                            return Err("unbalanced `{` (write `{{` for a literal brace)".into())
                        }
                        Some(ch) => name.push(ch),
                    }
                }
                if !lit.is_empty() {
                    out.push(Tok::Lit(std::mem::take(&mut lit)));
                }
                if let Some(t) = name.strip_prefix("$^") {
                    if !is_ident(t) {
                        return Err(format!("bad term name in `{{$^{t}}}`"));
                    }
                    out.push(Tok::Term(t.to_string(), Case::Title));
                } else if let Some(t) = name.strip_prefix('$') {
                    let lower = t.to_lowercase();
                    let case = if t == lower {
                        Case::As
                    } else if t.chars().skip(1).all(|c| !c.is_lowercase()) && t.chars().count() > 1
                    {
                        Case::Upper
                    } else if t.chars().skip(1).collect::<String>()
                        == lower.chars().skip(1).collect::<String>()
                    {
                        Case::Cap
                    } else {
                        return Err(format!("bad term token `{{${t}}}`"));
                    };
                    if !is_ident(&lower) {
                        return Err(format!("bad term name in `{{${t}}}`"));
                    }
                    out.push(Tok::Term(lower, case));
                } else {
                    if !is_ident(&name) {
                        return Err(format!(
                            "bad placeholder `{{{name}}}` (lowercase snake_case)"
                        ));
                    }
                    out.push(Tok::Arg(name));
                }
            }
            c => lit.push(c),
        }
    }
    if !lit.is_empty() {
        out.push(Tok::Lit(lit));
    }
    Ok(out)
}

fn parse_form(v: &Value, ctx: &str, errs: &mut Vec<String>) -> Option<Form> {
    let tok = |s: &str, errs: &mut Vec<String>| match tokenize(s) {
        Ok(t) => Some(t),
        Err(e) => {
            errs.push(format!("{ctx}: {e}"));
            None
        }
    };
    match v {
        Value::String(s) => {
            let t = tok(s, errs)?;
            Some(Form {
                git: t.clone(),
                local: t,
            })
        }
        Value::Table(t) => {
            let (Some(Value::String(g)), Some(Value::String(l)), 2) =
                (t.get("git"), t.get("local"), t.len())
            else {
                errs.push(format!(
                    "{ctx}: a table value needs exactly string `git` and `local` (or `one` and `other`)"
                ));
                return None;
            };
            Some(Form {
                git: tok(g, errs)?,
                local: tok(l, errs)?,
            })
        }
        _ => {
            errs.push(format!("{ctx}: a message is a string or a table"));
            None
        }
    }
}

fn parse_msg(v: &Value, ctx: &str, errs: &mut Vec<String>) -> Option<Msg> {
    if let Value::Table(t) = v {
        if t.contains_key("one") || t.contains_key("other") {
            let known = ["one", "few", "many", "other"];
            if !t.contains_key("one")
                || !t.contains_key("other")
                || t.keys().any(|k| !known.contains(&k.as_str()))
            {
                errs.push(format!(
                    "{ctx}: a plural has `one` and `other`, optionally `few` and `many`"
                ));
                return None;
            }
            let one = parse_form(&t["one"], &format!("{ctx}.one"), errs)?;
            let other = parse_form(&t["other"], &format!("{ctx}.other"), errs)?;
            let few = match t.get("few") {
                Some(v) => Some(parse_form(v, &format!("{ctx}.few"), errs)?),
                None => None,
            };
            let many = match t.get("many") {
                Some(v) => Some(parse_form(v, &format!("{ctx}.many"), errs)?),
                None => None,
            };
            return Some(Msg {
                plural: true,
                forms: vec![one, other],
                few,
                many,
            });
        }
    }
    Some(Msg {
        plural: false,
        forms: vec![parse_form(v, ctx, errs)?],
        few: None,
        many: None,
    })
}

fn parse_term_val(v: &Value) -> Option<TermVal> {
    match v {
        Value::String(s) => Some(TermVal::One(s.clone())),
        Value::Table(t) if t.len() == 2 => match (t.get("pull"), t.get("merge")) {
            (Some(Value::String(p)), Some(Value::String(m))) => Some(TermVal::Host {
                pull: p.clone(),
                merge: m.clone(),
            }),
            _ => None,
        },
        _ => None,
    }
}

fn load(dir: &Path, stem: &str, errs: &mut Vec<String>) -> Option<Catalog> {
    let path = dir.join(format!("{stem}.toml"));
    let text = fs::read_to_string(&path).ok()?;
    let file = format!("locales/{stem}.toml");
    let table: Table = match text.parse() {
        Ok(t) => t,
        Err(e) => {
            errs.push(format!("{file}: {e}"));
            return None;
        }
    };
    let mut cat = Catalog {
        name: String::new(),
        msgs: BTreeMap::new(),
        terms: BTreeMap::new(),
        english_option: matches!(stem, "ja" | "ko" | "zh-CN"),
    };
    for (section, body) in &table {
        let Value::Table(body) = body else {
            errs.push(format!(
                "{file}: top-level `{section}` must be a [section] table"
            ));
            continue;
        };
        match section.as_str() {
            "_meta" => match body.get("language_name") {
                Some(Value::String(s)) if !s.trim().is_empty() => cat.name = s.clone(),
                _ => errs.push(format!("{file}: [_meta] needs a non-empty `language_name`")),
            },
            "term" => {
                for (name, v) in body {
                    let ok = is_ident(name)
                        && v.as_table().is_some_and(|t| t.len() == 2)
                        && match (
                            v.get("git").and_then(parse_term_val),
                            v.get("local").and_then(parse_term_val),
                        ) {
                            (Some(g), Some(l)) => {
                                let same = matches!(
                                    (&g, &l),
                                    (TermVal::One(_), TermVal::One(_))
                                        | (TermVal::Host { .. }, TermVal::Host { .. })
                                );
                                if same {
                                    cat.terms.insert(name.clone(), Term { git: g, local: l });
                                }
                                same
                            }
                            _ => false,
                        };
                    if !ok {
                        errs.push(format!(
                            "{file}: [term] `{name}` must be {{ git = X, local = X }} where X is a string or {{ pull, merge }}, both the same kind"
                        ));
                    }
                }
            }
            _ => {
                if !is_ident(section) {
                    errs.push(format!("{file}: bad section name `[{section}]`"));
                }
                for (key, v) in body {
                    let ctx = format!("{file}: {section}.{key}");
                    if !is_ident(key) {
                        errs.push(format!(
                            "{ctx}: bad key (lowercase snake_case, not a Rust keyword)"
                        ));
                        continue;
                    }
                    if let Some(m) = parse_msg(v, &ctx, errs) {
                        cat.msgs.insert(format!("{section}.{key}"), m);
                    }
                }
            }
        }
    }
    if !table.contains_key("_meta") {
        errs.push(format!("{file}: missing [_meta] language_name"));
    }
    Some(cat)
}

fn args_of(f: &Form) -> BTreeSet<String> {
    f.git
        .iter()
        .chain(&f.local)
        .filter_map(|t| {
            if let Tok::Arg(a) = t {
                Some(a.clone())
            } else {
                None
            }
        })
        .collect()
}

fn terms_of(f: &Form) -> BTreeSet<String> {
    f.git
        .iter()
        .chain(&f.local)
        .filter_map(|t| {
            if let Tok::Term(a, _) = t {
                Some(a.clone())
            } else {
                None
            }
        })
        .collect()
}

/// Parameter list of an English message, in order of first appearance;
/// `n` leads a plural.
fn params_of(m: &Msg) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    if m.plural {
        out.push("n".into());
    }
    for (_, f) in m.all_forms() {
        for t in f.git.iter().chain(&f.local) {
            if let Tok::Arg(a) = t {
                if !out.contains(a) {
                    out.push(a.clone());
                }
            }
        }
    }
    out
}

fn validate(stem: &str, cat: &Catalog, en: &Catalog, errs: &mut Vec<String>) {
    let file = format!("locales/{stem}.toml");
    for (name, t) in &cat.terms {
        match en.terms.get(name) {
            None => errs.push(format!("{file}: [term] `{name}` is not in en.toml")),
            Some(e) => {
                if matches!(e.git, TermVal::One(_)) != matches!(t.git, TermVal::One(_)) {
                    errs.push(format!(
                        "{file}: [term] `{name}` has a different shape than in en.toml"
                    ));
                }
            }
        }
    }
    for (key, m) in &cat.msgs {
        let ctx = format!("{file}: {key}");
        for (_, f) in m.all_forms() {
            for t in terms_of(f) {
                if !en.terms.contains_key(&t) {
                    errs.push(format!(
                        "{ctx}: unknown term `{{${t}}}` (define it in [term] of en.toml)"
                    ));
                }
            }
        }
        let Some(em) = en.msgs.get(key) else {
            if stem != "en" {
                errs.push(format!(
                    "{ctx}: key does not exist in en.toml (stale or misspelled)"
                ));
            }
            continue;
        };
        if em.plural != m.plural {
            errs.push(format!(
                "{ctx}: plural in one language but not in the other (en.toml differs)"
            ));
            continue;
        }
        let allowed: BTreeSet<String> = em.forms.iter().flat_map(args_of).collect();
        for (name, f) in m.all_forms() {
            for (side, toks) in [("git", &f.git), ("local", &f.local)] {
                let set: BTreeSet<String> = toks
                    .iter()
                    .filter_map(|t| {
                        if let Tok::Arg(a) = t {
                            Some(a.clone())
                        } else {
                            None
                        }
                    })
                    .collect();
                let mut required = allowed.clone();
                if m.plural && name == "one" {
                    required.remove("n");
                }
                if !(required.is_subset(&set) && set.is_subset(&allowed)) {
                    errs.push(format!(
                        "{ctx} ({side}, form {name}): placeholders {set:?} do not match English {allowed:?}"
                    ));
                }
            }
        }
        for (name, f) in m.all_forms() {
            if m.plural && name != "one" && !args_of(f).contains("n") {
                errs.push(format!(
                    "{ctx}.{name}: a plural's `{name}` must contain {{n}}"
                ));
            }
        }
    }
    if let Some(e) = en
        .msgs
        .iter()
        .find(|(_, m)| m.plural && !args_of(&m.forms[1]).contains("n"))
    {
        if stem == "en" {
            errs.push(format!("{file}: {}.other must contain {{n}}", e.0));
        }
    }
}

/// Uppercase with the Turkish mapping (i -> İ, ı -> I) when `tr` is set.
fn upper(s: &str, tr: bool) -> String {
    if tr {
        s.replace('i', "İ").replace('ı', "I").to_uppercase()
    } else {
        s.to_uppercase()
    }
}

fn apply_case(s: &str, case: Case, tr: bool) -> String {
    match case {
        Case::As => s.to_string(),
        Case::Upper => upper(s, tr),
        Case::Title => s
            .split(' ')
            .map(|w| apply_case(w, Case::Cap, tr))
            .collect::<Vec<_>>()
            .join(" "),
        Case::Cap => {
            let mut c = s.chars();
            match c.next() {
                Some(f) => upper(&f.to_string(), tr).chars().chain(c).collect(),
                None => String::new(),
            }
        }
    }
}

fn term_text(name: &str, cat: &Catalog, en: &Catalog, variant: usize) -> String {
    let t = cat
        .terms
        .get(name)
        .or_else(|| en.terms.get(name))
        .expect("validated term");
    // Bit 2 (English forced, ja/ko/zh-CN): the English git form of the term.
    let t = if variant & 4 != 0 && cat.english_option {
        en.terms.get(name).unwrap_or(t)
    } else {
        t
    };
    let v = if variant & 2 != 0 { &t.local } else { &t.git };
    match v {
        TermVal::One(s) => s.clone(),
        TermVal::Host { pull, merge } => if variant & 1 != 0 { merge } else { pull }.clone(),
    }
}

fn resolve(
    m: &Msg,
    params: &[String],
    cat: &Catalog,
    en: &Catalog,
    variant: usize,
    form: usize,
) -> Vec<Seg> {
    // form: 0 one, 1 other, 2 few, 3 many (few and many fall back to other).
    let f = match form {
        2 => m.few.as_ref().unwrap_or(&m.forms[m.forms.len() - 1]),
        3 => m.many.as_ref().unwrap_or(&m.forms[m.forms.len() - 1]),
        _ => &m.forms[form.min(m.forms.len() - 1)],
    };
    let toks = if variant & 2 != 0 { &f.local } else { &f.git };
    // Turkish casing applies to the translated vocabulary only: the English
    // terms (`Commit`) keep the plain mapping.
    let tr = cat.name == "Türkçe" && variant & 2 != 0;
    let mut out: Vec<Seg> = Vec::new();
    let push = |s: String, out: &mut Vec<Seg>| match out.last_mut() {
        Some(Seg::Lit(l)) => l.push_str(&s),
        _ => out.push(Seg::Lit(s)),
    };
    for t in toks {
        match t {
            Tok::Lit(s) => push(s.clone(), &mut out),
            Tok::Term(n, case) => push(
                apply_case(&term_text(n, cat, en, variant), *case, tr),
                &mut out,
            ),
            Tok::Arg(a) => out.push(Seg::Arg(
                params.iter().position(|p| p == a).expect("validated"),
            )),
        }
    }
    out
}

const PSEUDO_UP: &str = "ÅƁĊĎĒƑĜĤĪĴĶĻṀŃŌƤǪŖŜŢŪṼŴẊŶŽ";
const PSEUDO_LO: &str = "åƀċďēƒĝĥīĵķļṁńōƥǫŕŝţūṽŵẋŷž";

fn pseudo(segs: Vec<Seg>) -> Vec<Seg> {
    let up: Vec<char> = PSEUDO_UP.chars().collect();
    let lo: Vec<char> = PSEUDO_LO.chars().collect();
    let mut chars = 0usize;
    let mut out: Vec<Seg> = segs
        .into_iter()
        .map(|s| match s {
            Seg::Lit(l) => Seg::Lit(
                l.chars()
                    .map(|c| {
                        chars += 1;
                        match c {
                            'A'..='Z' => up[c as usize - 'A' as usize],
                            'a'..='z' => lo[c as usize - 'a' as usize],
                            c => c,
                        }
                    })
                    .collect(),
            ),
            a => a,
        })
        .collect();
    let pad = (chars * 35).div_ceil(100).max(1);
    let tail = format!(" {}⟧", "·".repeat(pad));
    match out.first_mut() {
        Some(Seg::Lit(l)) => l.insert(0, '⟦'),
        _ => out.insert(0, Seg::Lit("⟦".into())),
    }
    match out.last_mut() {
        Some(Seg::Lit(l)) => l.push_str(&tail),
        _ => out.push(Seg::Lit(tail)),
    }
    out
}

fn lit(s: &str) -> String {
    format!("{s:?}")
}

/// Deduplicated strings; the generated `BLOB` is their concatenation in id
/// order, so string `i` spans `STR_OFFS[i]..STR_OFFS[i + 1]`.
#[derive(Default)]
struct Pool {
    ids: HashMap<String, u32>,
    strs: Vec<String>,
}

impl Pool {
    fn str_id(&mut self, s: &str) -> u32 {
        if let Some(&i) = self.ids.get(s) {
            return i;
        }
        let i = self.strs.len() as u32;
        self.ids.insert(s.to_string(), i);
        self.strs.push(s.to_string());
        i
    }
}

/// `# context:` comment blocks directly above a key in en.toml.
fn contexts(text: &str) -> BTreeMap<String, String> {
    let mut out = BTreeMap::new();
    let mut section = String::new();
    let mut pending: Vec<String> = Vec::new();
    for line in text.lines() {
        let l = line.trim();
        if let Some(c) = l.strip_prefix('#') {
            pending.push(c.trim().to_string());
        } else if l.starts_with('[') {
            section = l.trim_matches(|c| c == '[' || c == ']').to_string();
            pending.clear();
        } else if let Some((k, _)) = l.split_once('=') {
            if pending.first().is_some_and(|p| p.starts_with("context:")) {
                out.insert(format!("{section}.{}", k.trim()), pending.join("\n"));
            }
            pending.clear();
        } else {
            pending.clear();
        }
    }
    out
}

fn main() {
    println!("cargo:rerun-if-changed=locales");
    println!("cargo:rerun-if-changed=build.rs");
    let dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap()).join("locales");
    let mut errs = Vec::new();
    // Only built languages are loaded: an unbuilt one costs no build time and
    // none of its text reaches the tables (it resolves to the English row).
    let built: Vec<bool> = (0..N_LANGS).map(is_built).collect();
    let cats: Vec<Option<Catalog>> = LANGS
        .iter()
        .enumerate()
        .map(|(i, (s, _))| {
            if built[i] {
                load(&dir, s, &mut errs)
            } else {
                None
            }
        })
        .collect();
    // Row of each language in the generated tables; unbuilt ones share row 0
    // (English). `rows` lists the built languages in row order.
    let mut lang_row = [0u8; N_LANGS];
    let mut rows: Vec<usize> = Vec::new();
    for (l, b) in built.iter().enumerate() {
        if *b {
            lang_row[l] = rows.len() as u8;
            rows.push(l);
        }
    }
    let n_rows = rows.len();
    let Some(en) = cats[0].as_ref() else {
        errs.push("locales/en.toml is missing or unparsable".into());
        fail(&errs);
        unreachable!();
    };
    for (i, c) in cats.iter().enumerate() {
        if let Some(c) = c {
            validate(LANGS[i].0, c, en, &mut errs);
        }
    }
    fail(&errs);
    let en_text = fs::read_to_string(dir.join("en.toml")).unwrap();
    let docs = contexts(&en_text);

    // MISSING: keys (and terms) a present non-English file lacks.
    let mut missing: Vec<(usize, String)> = Vec::new();
    for (i, c) in cats.iter().enumerate().skip(1) {
        let Some(c) = c else { continue };
        let before = missing.len();
        for k in en.msgs.keys() {
            // Agents read English only: `agent_reason` lives in en.toml alone.
            if k.starts_with("agent_reason.") {
                continue;
            }
            if !c.msgs.contains_key(k) {
                missing.push((i, k.clone()));
            }
        }
        for t in en.terms.keys() {
            if !c.terms.contains_key(t) {
                missing.push((i, format!("term.{t}")));
            }
        }
        if missing.len() > before {
            println!(
                "cargo:warning=locales/{}.toml misses {} entries (English is used)",
                LANGS[i].0,
                missing.len() - before
            );
        }
    }

    let mut plain: Vec<(&String, &Msg)> = Vec::new();
    let mut args: Vec<(&String, &Msg, Vec<String>)> = Vec::new();
    for (k, m) in &en.msgs {
        let p = params_of(m);
        if p.is_empty() {
            plain.push((k, m));
        } else {
            args.push((k, m, p));
        }
    }
    // Which catalog and message serves `lang` for `key` (fallback: English).
    let pick = |lang: usize, key: &str| -> (&Msg, &Catalog) {
        if lang < QPS {
            if let Some(c) = cats[lang].as_ref() {
                if let Some(m) = c.msgs.get(key) {
                    return (m, c);
                }
            }
        }
        (&en.msgs[key], en)
    };
    // Segments of (lang, variant, form); qps is pseudo-localised English.
    let segs_of = |lang: usize, key: &str, params: &[String], variant: usize, form: usize| {
        let (m, c) = pick(lang, key);
        let s = resolve(m, params, c, en, variant, form);
        if lang == QPS {
            pseudo(s)
        } else {
            s
        }
    };
    let flat = |s: &[Seg]| -> String {
        s.iter()
            .map(|x| if let Seg::Lit(l) = x { l.as_str() } else { "" })
            .collect()
    };

    let mut o = String::new();
    let mut pool = Pool::default();
    writeln!(o, "pub(crate) const BUILT_LANGS: usize = {n_rows};").unwrap();
    writeln!(
        o,
        "pub(crate) const LANG_ROW: [u8; {N_LANGS}] = {lang_row:?};"
    )
    .unwrap();
    // Plain table layout: 1 slot, or 8 when any language varies by variant.
    let mut base = 0usize;
    let mut plain_ids: Vec<(usize, bool)> = Vec::new();
    for (k, _) in &plain {
        let varies = rows.iter().any(|&l| {
            let first = flat(&segs_of(l, k, &[], 0, 0));
            (1..N_VARIANTS).any(|v| flat(&segs_of(l, k, &[], v, 0)) != first)
        });
        plain_ids.push((base, varies));
        base += if varies { N_VARIANTS } else { 1 };
    }
    assert!(base < 0x8000, "too many messages for a 15-bit index");
    // PLAIN[lang][slot] is a string id (see `Pool`).
    let mut plain_rows: Vec<Vec<u32>> = Vec::new();
    for &l in &rows {
        let mut row = Vec::with_capacity(base);
        for ((k, _), (_, varies)) in plain.iter().zip(&plain_ids) {
            for v in 0..if *varies { N_VARIANTS } else { 1 } {
                row.push(pool.str_id(&flat(&segs_of(l, k, &[], v, 0))));
            }
        }
        plain_rows.push(row);
    }

    // Argument messages: each distinct segment sequence ("alternative") is a
    // range of SEGS; an ArgMsg maps (lang, variant, form) to a global
    // alternative id, stored relative to `base`.
    let mut alt_ids: HashMap<Vec<u32>, u32> = HashMap::new();
    let mut alt_list: Vec<Vec<u32>> = Vec::new();
    let mut arg_code = String::new();
    for (k, _m, params) in &args {
        let mut map = Vec::new();
        for &l in &rows {
            for v in 0..N_VARIANTS {
                for f in 0..4 {
                    let enc: Vec<u32> = segs_of(l, k, params, v, f)
                        .iter()
                        .map(|s| match s {
                            Seg::Lit(t) => pool.str_id(t) << 1,
                            Seg::Arg(i) => (*i as u32) << 1 | 1,
                        })
                        .collect();
                    let id = *alt_ids.entry(enc.clone()).or_insert_with(|| {
                        alt_list.push(enc);
                        (alt_list.len() - 1) as u32
                    });
                    map.push(id);
                }
            }
        }
        assert!(alt_list.len() < 0x1_0000, "too many distinct alternatives");
        writeln!(arg_code, "    crate::ArgMsg {{ map: {map:?} }},").unwrap();
    }
    // Flatten alternatives.
    let mut segs: Vec<u32> = Vec::new();
    let mut alt_offs: Vec<u32> = Vec::new();
    for a in &alt_list {
        alt_offs.push(segs.len() as u32);
        segs.extend(a);
    }
    alt_offs.push(segs.len() as u32);

    let n_str = pool.strs.len();
    let plain_ty = if n_str < 0x1_0000 { "u16" } else { "u32" };
    // Blob + offsets.
    let mut blob = String::new();
    let mut offs: Vec<u32> = Vec::with_capacity(n_str + 1);
    for s in &pool.strs {
        offs.push(blob.len() as u32);
        blob.push_str(s);
    }
    offs.push(blob.len() as u32);
    writeln!(o, "static BLOB: &str = {};", lit(&blob)).unwrap();
    writeln!(o, "static STR_OFFS: [u32; {}] = {offs:?};", offs.len()).unwrap();
    writeln!(o, "static SEGS: [u32; {}] = {segs:?};", segs.len()).unwrap();
    writeln!(
        o,
        "static ALT_OFFS: [u32; {}] = {alt_offs:?};",
        alt_offs.len()
    )
    .unwrap();
    writeln!(
        o,
        "pub(crate) static PLAIN: [[{plain_ty}; {base}]; {n_rows}] = {plain_rows:?};"
    )
    .unwrap();
    writeln!(
        o,
        "pub(crate) static ARG_MSGS: [crate::ArgMsg; {}] = [\n{arg_code}];",
        args.len()
    )
    .unwrap();
    o.push_str(
        r#"
/// Text of string id `id`.
#[inline]
pub(crate) fn string(id: usize) -> &'static str {
    &BLOB[STR_OFFS[id] as usize..STR_OFFS[id + 1] as usize]
}

/// Segment codes of alternative `alt`: `id << 1` is literal string `id`,
/// `arg << 1 | 1` is argument `arg`.
#[inline]
pub(crate) fn alt(alt: usize) -> &'static [u32] {
    &SEGS[ALT_OFFS[alt] as usize..ALT_OFFS[alt + 1] as usize]
}

#[cfg(test)]
mod blob_tests {
    #[test]
    fn spans_are_ordered_and_on_char_boundaries() {
        assert_eq!(super::STR_OFFS[0], 0);
        assert_eq!(*super::STR_OFFS.last().unwrap() as usize, super::BLOB.len());
        for w in super::STR_OFFS.windows(2) {
            assert!(w[0] <= w[1]);
        }
        for &o in &super::STR_OFFS {
            assert!(super::BLOB.is_char_boundary(o as usize));
        }
        for &o in &super::ALT_OFFS {
            assert!(o as usize <= super::SEGS.len());
        }
    }
}
"#,
    );

    // The msg tree.
    let mut tree: BTreeMap<&str, String> = BTreeMap::new();
    let doc = |k: &str, m: &Msg| -> String {
        let mut d = String::new();
        let f = &m.forms[m.forms.len() - 1];
        let eng: String = f
            .git
            .iter()
            .map(|t| match t {
                Tok::Lit(l) => l.clone(),
                Tok::Arg(a) => format!("{{{a}}}"),
                Tok::Term(n, _) => format!("{{${n}}}"),
            })
            .collect();
        writeln!(d, "    #[doc = {}]", lit(&format!("English: `{eng}`"))).unwrap();
        if let Some(c) = docs.get(k) {
            writeln!(d, "    #[doc = {}]", lit(c)).unwrap();
        }
        d
    };
    for ((k, m), (b, varies)) in plain.iter().zip(&plain_ids) {
        let (sec, name) = k.split_once('.').unwrap();
        let id = *b | if *varies { 0x8000 } else { 0 };
        let e = tree.entry(sec).or_default();
        e.push_str(&doc(k, m));
        writeln!(
            e,
            "    pub const {}: crate::M = crate::M({id});",
            name.to_uppercase()
        )
        .unwrap();
    }
    for (i, (k, m, params)) in args.iter().enumerate() {
        let (sec, name) = k.split_once('.').unwrap();
        let sig: Vec<String> = params
            .iter()
            .enumerate()
            .map(|(j, p)| {
                if m.plural && j == 0 {
                    "n: u64".to_string()
                } else {
                    format!("{p}: impl ::core::fmt::Display")
                }
            })
            .collect();
        let refs: Vec<String> = params
            .iter()
            .map(|p| format!("&{p} as &dyn ::core::fmt::Display"))
            .collect();
        let e = tree.entry(sec).or_default();
        e.push_str(&doc(k, m));
        writeln!(
            e,
            "    pub fn {name}({}) -> String {{ crate::format_arg(&super::super::ARG_MSGS[{i}], {}, &[{}]) }}",
            sig.join(", "),
            if m.plural { "Some(n)" } else { "None" },
            refs.join(", ")
        )
        .unwrap();
    }
    o.push_str("pub mod msg {\n");
    for (sec, body) in &tree {
        writeln!(o, "pub mod {sec} {{\n{body}}}").unwrap();
    }
    o.push_str("}\n");

    // Catalog metadata for tests and tools.
    o.push_str("pub static PLAIN_KEYS: &[(&str, crate::M)] = &[\n");
    for ((k, _), (b, varies)) in plain.iter().zip(&plain_ids) {
        let id = *b | if *varies { 0x8000 } else { 0 };
        writeln!(o, "    ({}, crate::M({id})),", lit(k)).unwrap();
    }
    o.push_str("];\npub static ARG_KEYS: &[crate::catalog::ArgKey] = &[\n");
    for (i, (k, m, params)) in args.iter().enumerate() {
        let ps: Vec<String> = params.iter().map(|p| lit(p)).collect();
        writeln!(
            o,
            "    crate::catalog::ArgKey {{ key: {}, params: &[{}], plural: {}, index: {i} }},",
            lit(k),
            ps.join(","),
            m.plural
        )
        .unwrap();
    }
    o.push_str("];\npub static TERM_NAMES: &[&str] = &[");
    for t in en.terms.keys() {
        write!(o, "{},", lit(t)).unwrap();
    }
    o.push_str("];\npub static MISSING: &[(crate::Lang, &str)] = &[\n");
    for (l, k) in &missing {
        writeln!(o, "    (crate::Lang::ALL[{l}], {}),", lit(k)).unwrap();
    }
    o.push_str("];\n");
    write!(o, "pub(crate) static LANG_NAMES: [&str; {N_LANGS}] = [").unwrap();
    for (i, (_, default)) in LANGS.iter().enumerate() {
        let n = cats[i]
            .as_ref()
            .map(|c| c.name.as_str())
            .filter(|n| !n.is_empty())
            .unwrap_or(default);
        write!(o, "{},", lit(n)).unwrap();
    }
    o.push_str("\"Pseudo-locale\"];\n");
    // Complete catalogs that are built, in picker order.
    o.push_str("pub const SHIPPED: &[crate::Lang] = &[\n");
    for code in SHIPPED_ORDER {
        let l = LANGS.iter().position(|(s, _)| s == code).unwrap();
        let complete = !missing.iter().any(|(i, _)| *i == l);
        if built[l] && complete {
            writeln!(o, "    crate::Lang::ALL[{l}],").unwrap();
        }
    }
    o.push_str("];\n");
    let built_codes: Vec<&str> = (0..N_LANGS)
        .filter(|&l| built[l])
        .map(|l| if l == QPS { "qps" } else { LANGS[l].0 })
        .collect();
    // Dependents (`gitpoulpi-commands`' build.rs) read this as
    // `DEP_GITPOULPI_I18N_LANGS` (needs `links` in Cargo.toml).
    println!("cargo::metadata=langs={}", built_codes.join(","));

    let out = PathBuf::from(env::var("OUT_DIR").unwrap()).join("messages.rs");
    fs::write(out, o).unwrap();
}

fn fail(errs: &[String]) {
    if !errs.is_empty() {
        panic!("i18n catalog errors:\n  {}", errs.join("\n  "));
    }
}
