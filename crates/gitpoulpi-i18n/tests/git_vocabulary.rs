//! Catalog vocabulary lint. Maintainer rule: with "Translate git terms" off, every
//! git word stays the English git word in every language.
//!
//! Git mode (all non-en languages): when the English text of a message holds a
//! git word, the localized text must hold the same stem. Local mode (secondary):
//! when a language's local term differs from the git one, the localized text
//! must not use the English stem.
//!
//! Known violations live in `tests/git_vocabulary/<code>.toml` (`git_mode` and
//! `local_mode` key lists). The test fails on a new violation AND on a stale
//! entry (now clean), so the baselines only shrink. Seed or refresh them with
//! `GIT_VOCAB_SEED=1 cargo test -p gitpoulpi-i18n --test git_vocabulary -- --ignored --nocapture print_violations`.

//! Needs every language: runs with `all-langs` (the default).

#![cfg(feature = "all-langs")]

use std::collections::BTreeSet;
use std::fmt::Display;
use std::fs;
use std::path::PathBuf;

use gitpoulpi_i18n::catalog::{render_arg, ARG_KEYS, PLAIN_KEYS};
use gitpoulpi_i18n::{
    override_english_git_terms, override_git_terms, override_lang, override_merge_terms, Lang,
    SHIPPED,
};

/// What an agent reads is always English by design (see i18n.md), so it is not linted.
const AGENT_PREFIX: &str = "agent_reason.";

/// (English word, stem the localized text must keep).
const WORDS: &[(&str, &str)] = &[
    ("commit", "commit"),
    ("commits", "commit"),
    ("push", "push"),
    ("pull", "pull"),
    ("fetch", "fetch"),
    ("stash", "stash"),
    ("stashes", "stash"),
    ("tag", "tag"),
    ("tags", "tag"),
    ("merge", "merg"),
    ("merged", "merg"),
    ("merges", "merg"),
    ("merging", "merg"),
    ("rebase", "rebas"),
    ("rebased", "rebas"),
    ("rebasing", "rebas"),
    ("diff", "diff"),
    ("diffs", "diff"),
    ("stage", "stag"),
    ("staged", "stag"),
    ("stages", "stag"),
    ("staging", "stag"),
    ("unstage", "stag"),
    ("unstaged", "stag"),
    ("unstages", "stag"),
    ("unstaging", "stag"),
];

/// Term names (in `[term]`) per stem, to find which stems have a distinct
/// local form in a language.
const TERMS_OF_STEM: &[(&str, &[&str])] = &[
    ("commit", &["commit", "commits"]),
    ("push", &["push"]),
    ("pull", &["pull"]),
    ("fetch", &["fetch"]),
    ("stash", &["stash", "stashes"]),
    ("tag", &["tag", "tags"]),
    ("merg", &["merge"]),
    ("rebas", &["rebase"]),
    ("cherry-pick", &["cherry_pick"]),
    ("diff", &["diff"]),
    ("stag", &["stage", "unstage", "staged", "unstaged"]),
];

/// Transliterations accepted in git mode (maintainer answer, 2026-10-05): ja and ko
/// write the git words in their script. Real translations (zh 暂存, ko 병합,
/// ja 差分) are not accepted. zh-CN has none: its git mode stays English.
fn translits(lang: Lang, stem: &str) -> &'static [&'static str] {
    match (lang, stem) {
        (Lang::Ja, "commit") => &["コミット"],
        (Lang::Ja, "push") => &["プッシュ"],
        (Lang::Ja, "pull") => &["プル"],
        (Lang::Ja, "fetch") => &["フェッチ"],
        (Lang::Ja, "stash") => &["スタッシュ"],
        (Lang::Ja, "tag") => &["タグ"],
        (Lang::Ja, "merg") => &["マージ"],
        (Lang::Ja, "rebas") => &["リベース"],
        (Lang::Ja, "cherry-pick") => &["チェリーピック"],
        (Lang::Ja, "diff") => &["ディフ"],
        (Lang::Ja, "stag") => &["ステージ"],
        (Lang::Ko, "commit") => &["커밋"],
        (Lang::Ko, "push") => &["푸시"],
        (Lang::Ko, "pull") => &["풀"],
        (Lang::Ko, "fetch") => &["페치"],
        (Lang::Ko, "stash") => &["스태시"],
        (Lang::Ko, "tag") => &["태그"],
        (Lang::Ko, "merg") => &["머지"],
        (Lang::Ko, "rebas") => &["리베이스"],
        (Lang::Ko, "cherry-pick") => &["체리픽"],
        (Lang::Ko, "diff") => &["디프"],
        (Lang::Ko, "stag") => &["스테이징", "스테이지"],
        _ => &[],
    }
}

/// `has_stem`, or one of the accepted transliterations of the stem.
fn has_stem_or_translit(lang: Lang, text: &str, stem: &str) -> bool {
    has_stem(text, stem) || translits(lang, stem).iter().any(|t| text.contains(t))
}

/// Kana, ideographs and hangul run into the Latin word beside them without a
/// space ("Commitメッセージ"), so they split tokens.
fn is_cjk(c: char) -> bool {
    matches!(c, '\u{3040}'..='\u{30ff}' | '\u{3400}'..='\u{9fff}' | '\u{ac00}'..='\u{d7af}')
}

fn tokens(text: &str) -> Vec<String> {
    text.to_lowercase()
        .split(|c: char| !c.is_alphanumeric() || is_cjk(c))
        .filter(|t| !t.is_empty())
        .map(str::to_owned)
        .collect()
}

/// English git stems present in an English text.
fn english_stems(text: &str) -> BTreeSet<&'static str> {
    let mut out = BTreeSet::new();
    for t in tokens(text) {
        if let Some((_, stem)) = WORDS.iter().find(|(w, _)| *w == t) {
            out.insert(*stem);
        }
    }
    if text.to_lowercase().contains("cherry-pick") {
        out.insert("cherry-pick");
    }
    out
}

/// Does the localized text use `stem` as a word (short inflection allowed, so
/// "différence" does not count as "diff")?
fn has_stem(text: &str, stem: &str) -> bool {
    if stem == "cherry-pick" {
        return text.to_lowercase().contains(stem);
    }
    tokens(text).iter().any(|t| {
        [
            t.as_str(),
            t.strip_prefix("un").unwrap_or(t),
            t.strip_prefix("ge").unwrap_or(t),
        ]
        .iter()
        .any(|t| {
            t.strip_prefix(stem)
                .is_some_and(|rest| rest.chars().count() <= 4)
        })
    })
}

/// Does the localized text hold the English git word itself (a whole token such
/// as "tag", "tags", "tagged", "unstaged")? A local word that merely shares the
/// stem (sv "tagg", es "rebasar") is not the English word and does not count.
fn has_english_word(text: &str, stem: &str) -> bool {
    if stem == "cherry-pick" {
        return text.to_lowercase().contains(stem);
    }
    tokens(text).iter().any(|t| {
        let t = t.strip_prefix("un").unwrap_or(t);
        english_forms(stem).contains(&t)
    })
}

/// The English words (and plain inflections) of a stem.
fn english_forms(stem: &str) -> Vec<&'static str> {
    const EXTRA: &[(&str, &str)] = &[
        ("commit", "committed"),
        ("commit", "committing"),
        ("push", "pushed"),
        ("push", "pushing"),
        ("push", "pushes"),
        ("pull", "pulled"),
        ("pull", "pulling"),
        ("pull", "pulls"),
        ("fetch", "fetched"),
        ("fetch", "fetching"),
        ("fetch", "fetches"),
        ("stash", "stashed"),
        ("stash", "stashing"),
        ("tag", "tagged"),
        ("tag", "tagging"),
        ("diff", "diffed"),
    ];
    WORDS
        .iter()
        .filter(|(_, s)| *s == stem)
        .map(|(w, _)| *w)
        .chain(EXTRA.iter().filter(|(s, _)| *s == stem).map(|(_, w)| *w))
        .collect()
}

/// Every rendering of every key in `lang`: (key, text). The order is the same
/// for every language, so two languages can be compared index by index.
fn renderings(lang: Lang, git_terms: bool) -> Vec<(String, String)> {
    let _l = override_lang(lang);
    let _g = override_git_terms(git_terms);
    let mut out = Vec::new();
    for merge in [false, true] {
        let _m = override_merge_terms(merge);
        for (key, m) in PLAIN_KEYS {
            if key.starts_with(AGENT_PREFIX) {
                continue;
            }
            out.push((key.to_string(), m.get().to_string()));
        }
        for k in ARG_KEYS {
            if k.key.starts_with(AGENT_PREFIX) {
                continue;
            }
            let markers: Vec<String> = k.params.iter().map(|p| format!("<<{p}>>")).collect();
            let forms: &[Option<u64>] = if k.plural {
                &[Some(1), Some(2), Some(5)]
            } else {
                &[None]
            };
            for &n in forms {
                let num = n.map(|n| n.to_string());
                let mut args: Vec<&dyn Display> =
                    markers.iter().map(|m| m as &dyn Display).collect();
                if let Some(num) = &num {
                    args[0] = num as &dyn Display;
                }
                out.push((k.key.to_string(), render_arg(k.index, n, &args)));
            }
        }
    }
    out
}

/// Stems whose local term differs from the git term in `<code>.toml`.
fn local_differing_stems(lang: Lang) -> BTreeSet<&'static str> {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("locales")
        .join(format!("{}.toml", lang.code()));
    let text = fs::read_to_string(path).expect("locale file");
    let mut in_term = false;
    let mut out = BTreeSet::new();
    for line in text.lines() {
        let line = line.trim();
        if line.starts_with('[') {
            in_term = line == "[term]";
            continue;
        }
        if !in_term {
            continue;
        }
        let Some((name, rest)) = line.split_once(" = ") else {
            continue;
        };
        let (Some(g), Some(l)) = (rest.find("git = \""), rest.find("local = \"")) else {
            continue;
        };
        let quoted = |at: usize, key: &str| {
            let s = &rest[at + key.len()..];
            s[..s.find('"').unwrap_or(s.len())].to_string()
        };
        if quoted(g, "git = \"") == quoted(l, "local = \"") {
            continue;
        }
        for (stem, names) in TERMS_OF_STEM {
            if names.contains(&name.trim()) {
                out.insert(*stem);
            }
        }
    }
    out
}

/// Sorted, de-duplicated violating keys: (git mode, local mode).
fn violations(lang: Lang) -> (Vec<String>, Vec<String>) {
    let en = renderings(Lang::En, false);

    let git = renderings(lang, false);
    assert_eq!(git.len(), en.len());
    let mut git_bad = BTreeSet::new();
    for ((key, en_text), (_, text)) in en.iter().zip(&git) {
        if english_stems(en_text)
            .iter()
            .any(|s| !has_stem_or_translit(lang, text, s))
        {
            git_bad.insert(key.clone());
        }
    }

    let differing = local_differing_stems(lang);
    let local = renderings(lang, true);
    let mut local_bad = BTreeSet::new();
    for ((key, en_text), (_, text)) in en.iter().zip(&local) {
        if english_stems(en_text)
            .iter()
            .any(|s| differing.contains(s) && has_english_word(text, s))
        {
            local_bad.insert(key.clone());
        }
    }
    (
        git_bad.into_iter().collect(),
        local_bad.into_iter().collect(),
    )
}

/// Keys whose English-forced rendering (ja, ko, zh-CN with the option on) lacks
/// an English git stem: an inline literal where a `{$term}` token belongs.
fn english_forced_violations(lang: Lang) -> Vec<String> {
    let en = renderings(Lang::En, false);
    let forced = {
        let _e = override_english_git_terms(true);
        renderings(lang, false)
    };
    let mut bad = BTreeSet::new();
    for ((key, en_text), (_, text)) in en.iter().zip(&forced) {
        if english_stems(en_text).iter().any(|s| !has_stem(text, s)) {
            bad.insert(key.clone());
        }
    }
    bad.into_iter().collect()
}

fn baseline_path(lang: Lang) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("git_vocabulary")
        .join(format!("{}.toml", lang.code()))
}

fn render_baseline(git: &[String], local: &[String]) -> String {
    let list = |name: &str, keys: &[String]| {
        let mut s = format!("{name} = [\n");
        for k in keys {
            s.push_str(&format!("  \"{k}\",\n"));
        }
        s.push_str("]\n");
        s
    };
    format!("{}\n{}", list("git_mode", git), list("local_mode", local))
}

/// Minimal reader for the two key lists.
fn read_baseline(lang: Lang) -> (Vec<String>, Vec<String>) {
    let text = fs::read_to_string(baseline_path(lang))
        .unwrap_or_else(|e| panic!("missing baseline for {}: {e}", lang.code()));
    let (mut git, mut local) = (Vec::new(), Vec::new());
    let mut into_git = true;
    for line in text.lines().map(str::trim) {
        if line.starts_with("git_mode") {
            into_git = true;
        } else if line.starts_with("local_mode") {
            into_git = false;
        } else if line.starts_with('"') {
            let key = line.trim_matches(|c| c == '"' || c == ',').to_string();
            if into_git {
                git.push(key);
            } else {
                local.push(key);
            }
        }
    }
    git.sort();
    local.sort();
    (git, local)
}

fn diff_lists(code: &str, mode: &str, now: &[String], base: &[String], errors: &mut Vec<String>) {
    let now_set: BTreeSet<_> = now.iter().collect();
    let base_set: BTreeSet<_> = base.iter().collect();
    for k in now_set.difference(&base_set) {
        errors.push(format!(
            "{code} {mode}: NEW violation `{k}` (use the git word)"
        ));
    }
    for k in base_set.difference(&now_set) {
        errors.push(format!(
            "{code} {mode}: STALE baseline entry `{k}` (now clean, remove it from tests/git_vocabulary/{code}.toml)"
        ));
    }
}

#[test]
fn git_words_stay_english_in_git_mode_and_baselines_only_shrink() {
    let mut errors = Vec::new();
    for &lang in SHIPPED.iter().filter(|l| **l != Lang::En) {
        let (git, local) = violations(lang);
        let (base_git, base_local) = read_baseline(lang);
        diff_lists(lang.code(), "git_mode", &git, &base_git, &mut errors);
        diff_lists(lang.code(), "local_mode", &local, &base_local, &mut errors);
    }
    assert!(errors.is_empty(), "\n{}", errors.join("\n"));
}

/// The option is only offered for ja, ko and zh-CN, and there every git word
/// comes from a `{$term}` token, so forcing English leaves no native git word.
/// No baseline: this list is always empty.
#[test]
fn english_forced_rendering_has_the_english_git_words() {
    for lang in [Lang::Ja, Lang::Ko, Lang::ZhCn] {
        let bad = english_forced_violations(lang);
        assert!(
            bad.is_empty(),
            "{}: inline git words (use a {{$term}} token): {bad:?}",
            lang.code()
        );
    }
}

#[test]
fn the_matcher_behaves() {
    assert!(has_stem("Pull request", "pull"));
    assert!(has_stem("{n} Dateien gestaged", "stag"));
    assert!(has_stem("2 fichiers unstaged", "stag"));
    assert!(!has_stem("Différence", "diff"));
    assert!(!has_english_word("Skapa tagg", "tag"));
    assert!(!has_english_word("rebasar la rama", "rebas"));
    assert!(has_english_word("Create tag here", "tag"));
    assert!(has_english_word("2 unstaged Dateien", "stag"));
    assert!(!has_stem("{n} fichiers indexés", "stag"));
    assert!(english_stems("Stage all and merge").contains("merg"));
    assert!(english_stems("Cherry-pick this").contains("cherry-pick"));
    assert!(english_stems("Staged 2 files").contains("stag"));
    assert!(english_stems("Open the stagecoach").is_empty());
    assert!(has_stem_or_translit(Lang::Ja, "全てをステージ", "stag"));
    assert!(has_stem_or_translit(Lang::Ko, "리베이스", "rebas"));
    assert!(!has_stem_or_translit(Lang::ZhCn, "全部暂存", "stag"));
    assert!(!has_stem_or_translit(Lang::Ko, "병합", "merg"));
}

#[test]
fn files_staged_is_clean_in_fr_and_es() {
    for lang in [Lang::Fr, Lang::Es] {
        let (git, local) = violations(lang);
        assert!(!git.contains(&"common.files_staged".to_string()), "{git:?}");
        assert!(
            !local.contains(&"common.files_staged".to_string()),
            "{local:?}"
        );
    }
}

/// French is fully converted: every git word follows the setting, so its
/// git-mode baseline stays empty.
#[test]
fn french_has_no_git_mode_violation() {
    let (git, _) = violations(Lang::Fr);
    assert!(git.is_empty(), "{git:?}");
    assert!(read_baseline(Lang::Fr).0.is_empty());
}

/// Prints the current violations as baseline files; with `GIT_VOCAB_SEED=1` it
/// also writes them. Run with `-- --ignored --nocapture print_violations`.
#[test]
#[ignore = "helper: prints (and with GIT_VOCAB_SEED=1 writes) the current baselines"]
fn print_violations() {
    let write = std::env::var_os("GIT_VOCAB_SEED").is_some();
    for &lang in SHIPPED.iter().filter(|l| **l != Lang::En) {
        let (git, local) = violations(lang);
        let body = render_baseline(&git, &local);
        println!(
            "=== {} (git {}, local {})\n{body}",
            lang.code(),
            git.len(),
            local.len()
        );
        if write {
            fs::write(baseline_path(lang), body).expect("write baseline");
        }
    }
}

/// Helper: `GIT_VOCAB_LANG=ja cargo test ... -- --ignored --nocapture print_forced`
/// lists the git-mode and English-forced offenders with their text.
#[test]
#[ignore = "helper: prints offending texts for one of ja, ko, zh-CN"]
fn print_forced() {
    let code = std::env::var("GIT_VOCAB_LANG").unwrap_or_else(|_| "ja".into());
    let lang = *SHIPPED.iter().find(|l| l.code() == code).expect("lang");
    let en = renderings(Lang::En, false);
    let git = renderings(lang, false);
    let forced = {
        let _e = override_english_git_terms(true);
        renderings(lang, false)
    };
    let mut seen = BTreeSet::new();
    for (((key, en_text), (_, g)), (_, f)) in en.iter().zip(&git).zip(&forced) {
        let stems = english_stems(en_text);
        let gb = stems.iter().any(|s| !has_stem_or_translit(lang, g, s));
        let fb = stems.iter().any(|s| !has_stem(f, s));
        if (gb || fb) && seen.insert(key.clone()) {
            println!(
                "{key}\t{}\tEN: {en_text}\tGIT: {g}\tFORCED: {f}",
                if gb { "G" } else { "F" }
            );
        }
    }
}

/// Helper: dumps every local-mode rendering of one language (to prove a catalog
/// rewrite left the translated vocabulary untouched).
#[test]
#[ignore = "helper"]
fn dump_local() {
    let code = std::env::var("GIT_VOCAB_LANG").unwrap_or_else(|_| "ja".into());
    let lang = *SHIPPED.iter().find(|l| l.code() == code).expect("lang");
    for (k, t) in renderings(lang, true) {
        println!("@@{k}\t{t}");
    }
}

#[test]
fn ja_ko_zh_have_no_git_mode_violation() {
    for lang in [Lang::Ja, Lang::Ko, Lang::ZhCn] {
        let (git, _) = violations(lang);
        assert!(git.is_empty(), "{}: {git:?}", lang.code());
        assert!(read_baseline(lang).0.is_empty());
    }
}

/// The English-words option swaps the stage terms in ja, ko and zh-CN only,
/// and only while the translated vocabulary is off.
#[test]
fn the_english_option_swaps_the_git_form_in_ja_ko_zh_only() {
    use gitpoulpi_i18n::msg::footer_hint::STAGE;
    let _e = override_english_git_terms(true);
    for (lang, git_form, local_form) in [
        (Lang::Ja, "stage", "ステージ"),
        (Lang::Ko, "stage", "스테이징"),
        (Lang::ZhCn, "stage", "暂存"),
    ] {
        let _l = override_lang(lang);
        let _off = override_english_git_terms(false);
        let _g = override_git_terms(false);
        let normal = STAGE.get().to_string();
        drop(_off);
        assert_eq!(STAGE.get(), git_form, "{} forced", lang.code());
        let _t = override_git_terms(true);
        assert_eq!(STAGE.get(), local_form, "{} translated", lang.code());
        assert!(
            normal == git_form || normal == local_form,
            "{normal}: {} off",
            lang.code()
        );
    }
    let _l = override_lang(Lang::Fr);
    let _g = override_git_terms(false);
    let off = {
        let _x = override_english_git_terms(false);
        STAGE.get()
    };
    assert_eq!(STAGE.get(), off, "fr ignores the option");
}
