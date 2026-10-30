//! Guards the generated table encoding against a reference implementation
//! that needs no pinned hash: every lookup (every language x vocabulary
//! variant, plain and argument messages, every plural category) is
//! recomputed straight from `locales/*.toml` and compared with the compiled
//! tables. The reference deliberately avoids `build.rs`'s segment/blob
//! machinery: it interpolates strings directly. Only meaningful with every
//! language built.

#![cfg(feature = "all-langs")]

use std::collections::HashMap;
use std::fmt::Write as _;
use std::path::Path;

use gitpoulpi_i18n::catalog::{render_arg, ARG_KEYS, PLAIN_KEYS};
use gitpoulpi_i18n::{
    override_english_git_terms, override_git_terms, override_lang, override_merge_terms, Lang,
    Plural,
};
use toml::{Table, Value};

type Cats = HashMap<&'static str, Table>;

fn load() -> Cats {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("locales");
    Lang::ALL
        .into_iter()
        .filter(|l| *l != Lang::Qps)
        .map(|l| {
            let text = std::fs::read_to_string(dir.join(format!("{}.toml", l.code()))).unwrap();
            (l.code(), text.parse::<Table>().unwrap())
        })
        .collect()
}

fn message<'a>(cat: &'a Table, key: &str) -> Option<&'a Value> {
    let (section, name) = key.split_once('.').unwrap();
    cat.get(section)?.as_table()?.get(name)
}

/// A piece of a rendered message: literal text, or argument `i`.
enum Piece {
    Lit(String),
    Arg(usize),
}

fn upper(s: &str, tr: bool) -> String {
    if tr {
        s.replace('i', "İ").replace('ı', "I").to_uppercase()
    } else {
        s.to_uppercase()
    }
}

fn cap(s: &str, tr: bool) -> String {
    let mut c = s.chars();
    c.next()
        .map(|f| upper(&f.to_string(), tr) + c.as_str())
        .unwrap_or_default()
}

/// Text of term `name` for `variant` as served by catalog `stem`.
fn term(cats: &Cats, stem: &str, name: &str, variant: usize) -> String {
    let en = &cats["en"]["term"][name];
    let own = cats[stem].get("term").and_then(|t| t.get(name));
    let mut t = own.unwrap_or(en);
    if variant & 4 != 0 && matches!(stem, "ja" | "ko" | "zh-CN") {
        t = en;
    }
    let v = &t[if variant & 2 != 0 { "local" } else { "git" }];
    match v {
        Value::String(s) => s.clone(),
        Value::Table(h) => h[if variant & 1 != 0 { "merge" } else { "pull" }]
            .as_str()
            .unwrap()
            .to_string(),
        _ => panic!("bad term {name}"),
    }
}

fn expand(cats: &Cats, stem: &str, text: &str, params: &[&str], variant: usize) -> Vec<Piece> {
    let tr = stem == "tr" && variant & 2 != 0;
    let mut out = Vec::new();
    let mut lit = String::new();
    let mut rest = text;
    while let Some(i) = rest.find(['{', '}']) {
        lit.push_str(&rest[..i]);
        let tail = &rest[i..];
        if tail.starts_with("{{") || tail.starts_with("}}") {
            lit.push_str(&tail[..1]);
            rest = &tail[2..];
            continue;
        }
        assert!(tail.starts_with('{'), "stray brace in {text:?}");
        let end = tail.find('}').unwrap();
        let name = &tail[1..end];
        rest = &tail[end + 1..];
        if let Some(t) = name.strip_prefix('$') {
            let (t, title) = t.strip_prefix('^').map_or((t, false), |t| (t, true));
            let text = term(cats, stem, &t.to_lowercase(), variant);
            let shown = if title {
                text.split(' ')
                    .map(|w| cap(w, tr))
                    .collect::<Vec<_>>()
                    .join(" ")
            } else if t == t.to_lowercase() {
                text
            } else if t.chars().count() > 1 && t == t.to_uppercase() {
                upper(&text, tr)
            } else {
                cap(&text, tr)
            };
            lit.push_str(&shown);
        } else {
            if !lit.is_empty() {
                out.push(Piece::Lit(std::mem::take(&mut lit)));
            }
            out.push(Piece::Arg(params.iter().position(|p| *p == name).unwrap()));
        }
    }
    lit.push_str(rest);
    if !lit.is_empty() {
        out.push(Piece::Lit(lit));
    }
    out
}

/// Pseudo-locale: accents on ASCII letters, bracketed and padded by 35 %.
fn pseudo(pieces: Vec<Piece>) -> Vec<Piece> {
    const UP: &str = "ÅƁĊĎĒƑĜĤĪĴĶĻṀŃŌƤǪŖŜŢŪṼŴẊŶŽ";
    const LO: &str = "åƀċďēƒĝĥīĵķļṁńōƥǫŕŝţūṽŵẋŷž";
    let (up, lo): (Vec<char>, Vec<char>) = (UP.chars().collect(), LO.chars().collect());
    let mut chars = 0;
    let mut out: Vec<Piece> = pieces
        .into_iter()
        .map(|p| match p {
            Piece::Lit(l) => Piece::Lit(
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
    let pad = (chars * 35usize).div_ceil(100).max(1);
    out.insert(0, Piece::Lit("⟦".into()));
    out.push(Piece::Lit(format!(" {}⟧", "·".repeat(pad))));
    out
}

/// The expected text of `key` for `lang`/`variant`; `form` is the plural
/// category name (`None`: not a plural), `args` the rendered arguments.
fn expected(
    cats: &Cats,
    lang: Lang,
    variant: usize,
    key: &str,
    params: &[&str],
    form: Option<Plural>,
    args: &[String],
) -> String {
    let (stem, msg) = match message(cats.get(lang.code()).unwrap_or(&cats["en"]), key) {
        Some(m) if lang != Lang::Qps => (lang.code(), m),
        _ => ("en", message(&cats["en"], key).unwrap()),
    };
    let value = match (form, msg.as_table()) {
        (Some(f), Some(t)) if t.contains_key("one") => {
            let name = match f {
                Plural::One => "one",
                Plural::Few if t.contains_key("few") => "few",
                Plural::Many if t.contains_key("many") => "many",
                _ => "other",
            };
            &t[name]
        }
        _ => msg,
    };
    let text = match value {
        Value::String(s) => s,
        Value::Table(t) => t[if variant & 2 != 0 { "local" } else { "git" }]
            .as_str()
            .unwrap(),
        _ => panic!("{key}: bad value"),
    };
    let mut pieces = expand(cats, stem, text, params, variant);
    if lang == Lang::Qps {
        pieces = pseudo(pieces);
    }
    let mut s = String::new();
    for p in pieces {
        match p {
            Piece::Lit(l) => s.push_str(&l),
            Piece::Arg(i) => s.push_str(&args[i]),
        }
    }
    s
}

#[test]
fn every_lookup_matches_the_toml() {
    let cats = load();
    let mut count = 0usize;
    let mut bad = String::new();
    for l in Lang::ALL {
        for bits in 0..8u8 {
            let _a = override_lang(l);
            let _b = override_git_terms(bits & 2 != 0);
            let _c = override_merge_terms(bits & 1 != 0);
            let _d = override_english_git_terms(bits & 4 != 0);
            // The variant the library derives: English forcing needs the
            // English-capable languages and the untranslated vocabulary.
            let forced = bits & 4 != 0 && bits & 2 == 0 && l.has_english_git_option();
            let variant = usize::from(forced) << 2 | usize::from(bits & 3);
            let mut check = |what: String, got: String, want: String| {
                count += 1;
                if got != want && bad.len() < 4000 {
                    let _ = writeln!(bad, "{what}: got {got:?}, want {want:?}");
                }
            };
            for (key, m) in PLAIN_KEYS {
                let want = expected(&cats, l, variant, key, &[], None, &[]);
                check(format!("{l:?}/{bits}/{key}"), m.get().to_string(), want);
            }
            for k in ARG_KEYS {
                let strs: Vec<String> = (0..k.params.len()).map(|i| format!("<{i}>")).collect();
                let refs: Vec<&dyn std::fmt::Display> =
                    strs.iter().map(|a| a as &dyn std::fmt::Display).collect();
                if k.plural {
                    for n in [0u64, 1, 2, 5, 11, 12, 21, 22] {
                        let mut r = refs.clone();
                        r[0] = &n;
                        let mut a = strs.clone();
                        a[0] = n.to_string();
                        let want = expected(
                            &cats,
                            l,
                            variant,
                            k.key,
                            k.params,
                            Some(l.plural_form(n)),
                            &a,
                        );
                        let got = render_arg(k.index, Some(n), &r);
                        check(format!("{l:?}/{bits}/{}/n={n}", k.key), got, want);
                    }
                } else {
                    let want = expected(&cats, l, variant, k.key, k.params, None, &strs);
                    let got = render_arg(k.index, None, &refs);
                    check(format!("{l:?}/{bits}/{}", k.key), got, want);
                }
            }
        }
    }
    assert!(bad.is_empty(), "lookups differing from the TOML:\n{bad}");
    assert!(count > 100_000, "suspiciously few lookups: {count}");
}
