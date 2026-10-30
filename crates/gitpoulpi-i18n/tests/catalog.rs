//! Catalog guards. Structural checks (unknown keys, placeholder parity,
//! plural shape, brace balance, unknown terms) already fail the *build*;
//! these cover what only the generated data can show.
//! They need every language, so they run with `all-langs` (the default).

#![cfg(feature = "all-langs")]

use std::fmt::Display;

use gitpoulpi_i18n::catalog::{render_arg, ARG_KEYS, PLAIN_KEYS, TERM_NAMES};
use gitpoulpi_i18n::{
    current, epoch, month_abbr, month_name, msg, override_git_terms, override_lang,
    override_merge_terms, with_lang, Lang, Plural, MISSING, SHIPPED,
};

#[test]
fn shipped_set_starts_with_en_and_fr() {
    assert_eq!(&SHIPPED[..2], &[Lang::En, Lang::Fr]);
}

#[test]
fn every_key_and_term_is_present_in_shipped_languages() {
    for l in SHIPPED {
        let missing: Vec<_> = MISSING
            .iter()
            .filter(|(m, _)| m == l)
            .map(|(_, k)| *k)
            .collect();
        assert!(missing.is_empty(), "{} lacks {missing:?}", l.code());
    }
    assert!(TERM_NAMES.contains(&"commit") && TERM_NAMES.contains(&"pr"));
}

#[test]
fn plain_messages_are_non_empty_in_every_shipped_language_and_vocabulary() {
    for &l in SHIPPED {
        for git in [false, true] {
            for merge in [false, true] {
                let _a = override_lang(l);
                let _b = override_git_terms(git);
                let _c = override_merge_terms(merge);
                for (key, m) in PLAIN_KEYS {
                    assert!(!m.get().trim().is_empty(), "{key} empty in {}", l.code());
                }
            }
        }
    }
}

/// Renders every argument message with marker values and checks that each
/// placeholder survived (build.rs checks the sets; this checks the output).
#[test]
fn placeholders_survive_rendering_in_every_shipped_language() {
    for &l in SHIPPED {
        let _g = override_lang(l);
        for k in ARG_KEYS {
            let markers: Vec<String> = k.params.iter().map(|p| format!("<<{p}>>")).collect();
            let args: Vec<&dyn Display> = markers.iter().map(|m| m as &dyn Display).collect();
            let forms: &[Option<u64>] = if k.plural {
                &[Some(1), Some(5)]
            } else {
                &[None]
            };
            for &n in forms {
                // A plural's `n` parameter is a number; render with the count.
                let num = n.map(|n| n.to_string());
                let mut args = args.clone();
                if let Some(num) = &num {
                    args[0] = num as &dyn Display;
                }
                let out = render_arg(k.index, n, &args);
                for (i, p) in k.params.iter().enumerate() {
                    let needle = if k.plural && i == 0 {
                        num.clone().unwrap()
                    } else {
                        format!("<<{p}>>")
                    };
                    let optional_n = k.plural && i == 0 && n == Some(1);
                    assert!(
                        out.contains(&needle) || optional_n,
                        "{} ({}): `{out}` lost {{{p}}}",
                        k.key,
                        l.code()
                    );
                }
            }
        }
    }
}

#[test]
fn plural_rules_per_language() {
    use Plural::{Few, Many, One, Other};
    // (lang, [form of 0, 1, 2, 5, 21, 100])
    let cases = [
        (Lang::Nl, [Other, One, Other, Other, Other, Other]),
        (Lang::Sv, [Other, One, Other, Other, Other, Other]),
        (Lang::Tr, [Other, One, Other, Other, Other, Other]),
        (Lang::ZhCn, [Other, Other, Other, Other, Other, Other]),
        (Lang::Ja, [Other, Other, Other, Other, Other, Other]),
        (Lang::Ko, [Other, Other, Other, Other, Other, Other]),
        // ru/uk: one 1, 21, 31..; few 2-4, 22-24..; many 0, 5-20, 25-30..
        (Lang::Ru, [Many, One, Few, Many, One, Many]),
        (Lang::Uk, [Many, One, Few, Many, One, Many]),
        // pl: one only 1; few 2-4, 22-24..; many the rest (21 is many)
        (Lang::Pl, [Many, One, Few, Many, Many, Many]),
        // cs: one 1; few 2-4; other (many is for decimals) the rest
        (Lang::Cs, [Other, One, Few, Other, Other, Other]),
        (Lang::En, [Other, One, Other, Other, Other, Other]),
        (Lang::Es, [Other, One, Other, Other, Other, Other]),
        (Lang::It, [Other, One, Other, Other, Other, Other]),
        (Lang::De, [Other, One, Other, Other, Other, Other]),
        (Lang::PtPt, [Other, One, Other, Other, Other, Other]),
        (Lang::Fr, [One, One, Other, Other, Other, Other]),
        (Lang::PtBr, [One, One, Other, Other, Other, Other]),
    ];
    for (l, want) in cases {
        let got = [0, 1, 2, 5, 21, 100].map(|n| l.plural_form(n));
        assert_eq!(got, want, "{}", l.code());
    }
}

#[test]
fn slavic_plural_edge_cases() {
    // The teens are many, not few or one: 11-14, 111-114.
    for l in [Lang::Ru, Lang::Uk, Lang::Pl] {
        for n in [11, 12, 13, 14, 111, 112, 113, 114] {
            assert_eq!(l.plural_form(n), Plural::Many, "{} {n}", l.code());
        }
        for n in [22, 23, 24, 102, 103, 104, 1002] {
            assert_eq!(l.plural_form(n), Plural::Few, "{} {n}", l.code());
        }
    }
    for n in [31, 101, 1001] {
        assert_eq!(Lang::Ru.plural_form(n), Plural::One);
        assert_eq!(Lang::Uk.plural_form(n), Plural::One);
        assert_eq!(Lang::Pl.plural_form(n), Plural::Many);
    }
    assert_eq!(Lang::Cs.plural_form(5), Plural::Other);
    assert_eq!(Lang::Cs.plural_form(4), Plural::Few);
    assert_eq!(Lang::Cs.plural_form(22), Plural::Other);
}

#[test]
fn plural_messages_follow_the_language_rule() {
    with_lang(Lang::En, || {
        assert_eq!(msg::common::files_staged(0), "Staged 0 files");
        assert_eq!(msg::common::files_staged(1), "Staged 1 file");
        assert_eq!(msg::common::files_staged(2), "Staged 2 files");
    });
    with_lang(Lang::Fr, || {
        // Git words stay English by default; the setting switches to French.
        assert_eq!(msg::common::files_staged(0), "0 fichier staged");
        assert_eq!(msg::common::files_staged(1), "1 fichier staged");
        assert_eq!(msg::common::files_staged(2), "2 fichiers staged");
        let _t = override_git_terms(true);
        assert_eq!(msg::common::files_staged(0), "0 fichier indexé");
        assert_eq!(msg::common::files_staged(1), "1 fichier indexé");
        assert_eq!(msg::common::files_staged(2), "2 fichiers indexés");
        assert_eq!(msg::reltime::hours_ago(3), "il y a 3 heures");
    });
}

#[test]
fn placeholders_may_be_reordered_and_braces_escaped() {
    assert_eq!(msg::common::branch_renamed("a", "b"), "Renamed a to b");
    assert_eq!(msg::common::BRACES.get(), "Use {name} as a placeholder");
}

#[test]
fn git_terms_default_to_english_and_toggle_per_language() {
    // English everywhere by default, even in French.
    with_lang(Lang::Fr, || {
        assert_eq!(msg::common::PUSH.get(), "Push");
        assert_eq!(msg::common::STASH.get(), "Stash");
        let _t = override_git_terms(true);
        assert_eq!(msg::common::PUSH.get(), "Pousser");
        assert_eq!(msg::common::STASH.get(), "Remise");
    });
    // English has both vocabularies, identical.
    with_lang(Lang::En, || {
        let _t = override_git_terms(true);
        assert_eq!(msg::common::PUSH.get(), "Push");
    });
}

#[test]
fn per_vocabulary_text_overrides_token_expansion() {
    with_lang(Lang::Fr, || {
        assert_eq!(msg::common::UNDO_COMMIT.get(), "Annuler le commit");
        let _t = override_git_terms(true);
        assert_eq!(msg::common::UNDO_COMMIT.get(), "Annuler la validation");
    });
}

#[test]
fn terms_apply_inside_plurals_and_arguments() {
    with_lang(Lang::Fr, || {
        assert_eq!(msg::common::commits_ahead(3), "3 commits d'avance");
        let _t = override_git_terms(true);
        assert_eq!(msg::common::commits_ahead(3), "3 validations d'avance");
        assert_eq!(msg::common::commits_ahead(1), "1 validation d'avance");
    });
}

#[test]
fn pr_or_mr_follows_the_host_and_the_vocabulary() {
    with_lang(Lang::En, || {
        assert_eq!(msg::common::CREATE_PR.get(), "Create pull request");
        let _m = override_merge_terms(true);
        assert_eq!(msg::common::CREATE_PR.get(), "Create merge request");
    });
    with_lang(Lang::Fr, || {
        let _m = override_merge_terms(true);
        assert_eq!(msg::common::CREATE_PR.get(), "Créer une merge request");
        let _t = override_git_terms(true);
        assert_eq!(msg::common::CREATE_PR.get(), "Créer une demande de fusion");
    });
}

#[test]
fn unshipped_languages_fall_back_to_english() {
    for l in [Lang::Es, Lang::PtBr, Lang::PtPt, Lang::It, Lang::De]
        .into_iter()
        .filter(|l| !l.is_shipped())
    {
        with_lang(l, || {
            assert_eq!(msg::month::JANUARY.get(), "January", "{}", l.code());
            assert_eq!(msg::common::files_staged(2), "Staged 2 files");
        });
    }
}

#[test]
fn spanish_spot_checks() {
    with_lang(Lang::Es, || {
        assert_eq!(msg::month::JANUARY.get(), "enero");
        assert_eq!(msg::common::files_staged(1), "1 archivo staged");
        assert_eq!(msg::common::files_staged(0), "0 archivos staged");
        let _t = override_git_terms(true);
        assert_eq!(msg::common::files_staged(1), "1 archivo preparado");
        assert_eq!(msg::common::files_staged(0), "0 archivos preparados");
        assert_eq!(msg::toolbar::PULL.get(), "Traer");
    });
}

#[test]
fn endonyms() {
    assert_eq!(Lang::En.native_name(), "English");
    assert_eq!(Lang::Fr.native_name(), "Français");
    assert_eq!(Lang::PtBr.native_name(), "Português (Brasil)");
    assert_eq!(Lang::PtPt.native_name(), "Português (Portugal)");
    for (l, name) in [
        (Lang::ZhCn, "简体中文"),
        (Lang::Ja, "日本語"),
        (Lang::Ko, "한국어"),
        (Lang::Ru, "Русский"),
        (Lang::Uk, "Українська"),
        (Lang::Pl, "Polski"),
        (Lang::Cs, "Čeština"),
        (Lang::Sv, "Svenska"),
        (Lang::Tr, "Türkçe"),
        (Lang::Nl, "Nederlands"),
    ] {
        assert_eq!(l.native_name(), name, "{}", l.code());
    }
}

/// Wired languages that are not shipped yet (no complete file) draw English
/// and are kept out of the picker; shipping one is a `SHIPPED` edit.
#[test]
fn wired_but_unshipped_languages_show_english_and_are_not_listed() {
    for l in Lang::ALL {
        if l == Lang::Qps || l.is_shipped() {
            continue;
        }
        with_lang(l, || {
            assert_eq!(msg::month::JANUARY.get(), "January", "{}", l.code());
            assert_eq!(msg::common::files_staged(5), "Staged 5 files");
        });
    }
    assert!(!SHIPPED.contains(&Lang::Qps));
}

#[test]
fn every_real_language_is_either_shipped_or_listed_as_pending() {
    let pending = [
        Lang::Ru,
        Lang::Uk,
        Lang::Pl,
        Lang::Cs,
        Lang::Nl,
        Lang::Sv,
        Lang::Tr,
    ];
    for l in Lang::ALL {
        if l != Lang::Qps {
            assert!(
                l.is_shipped() || pending.contains(&l),
                "{} is neither shipped nor pending",
                l.code()
            );
        }
    }
}

#[test]
fn months_and_relative_dates() {
    with_lang(Lang::Fr, || {
        assert_eq!(month_name(2).get(), "février");
        assert_eq!(month_abbr(7).get(), "juil.");
        assert_eq!(month_name(0).get(), "janvier");
        assert_eq!(month_name(99).get(), "décembre");
        assert_eq!(msg::reltime::JUST_NOW.get(), "à l'instant");
    });
}

#[test]
fn pseudo_locale_wraps_and_pads() {
    with_lang(Lang::Qps, || {
        for (key, m) in PLAIN_KEYS {
            let en = m.english();
            let q = m.get();
            assert!(q.starts_with('⟦') && q.ends_with('⟧'), "{key}: {q}");
            let w = q.chars().count();
            let want = en.chars().count() * 135 / 100;
            assert!(w >= want, "{key}: {w} < {want}");
        }
        assert!(msg::common::files_staged(2).starts_with('⟦'));
        assert!(msg::common::files_staged(2).contains('2'));
    });
}

#[test]
fn thread_override_is_isolated_and_bumps_the_epoch() {
    let e0 = epoch();
    let g = override_lang(Lang::Fr);
    assert_ne!(epoch(), e0);
    assert_eq!(current(), Lang::Fr);
    std::thread::spawn(|| assert_eq!(current(), Lang::En))
        .join()
        .unwrap();
    let e1 = epoch();
    drop(g);
    assert_ne!(epoch(), e1);
    assert_eq!(current(), Lang::En);
}

#[test]
fn plain_lookups_are_static_strs() {
    // The type is the guarantee; this documents it.
    let s: &'static str = msg::common::PUSH.get();
    assert_eq!(s, "Push");
}

/// The maintainer wants the plain `'` apostrophe in every UI text, never the
/// typographic one (`l'IA`, not `l’IA`). Quotation marks such as `‘x’`
/// stay allowed: only a `’` glued to a following letter is an apostrophe.
#[test]
fn ui_and_cli_help_catalogs_use_the_plain_apostrophe() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let dirs = [
        root.join("locales"),
        root.join("../gitpoulpi-commands/locales/help"),
    ];
    let mut offenders = Vec::new();
    for dir in dirs {
        for entry in std::fs::read_dir(&dir).unwrap() {
            let path = entry.unwrap().path();
            if path.extension().is_none_or(|e| e != "toml") {
                continue;
            }
            let text = std::fs::read_to_string(&path).unwrap();
            for (n, line) in text.lines().enumerate() {
                let chars: Vec<char> = line.chars().collect();
                let glued = chars.windows(3).any(|w| {
                    w[1] == '\u{2019}'
                        && w[0].is_alphabetic()
                        && (w[2].is_alphabetic() || w[2] == '{')
                });
                // TOML spells it `\u2019` too.
                if glued || line.contains("\\u2019") {
                    offenders.push(format!("{}:{}", path.display(), n + 1));
                }
            }
        }
    }
    assert!(
        offenders.is_empty(),
        "typographic apostrophes: {offenders:?}"
    );
}

/// The noun forms of the git actions (`push_noun`...): a sentence that names
/// the operation ("LFS push: 2/5", "from before the pull") reads a noun in
/// the translated vocabulary while the git vocabulary keeps the English word.
#[test]
fn noun_terms_give_the_operation_its_noun_in_the_translated_vocabulary() {
    for name in [
        "push_noun",
        "pull_noun",
        "fetch_noun",
        "merge_noun",
        "rebase_noun",
    ] {
        assert!(TERM_NAMES.contains(&name), "{name} is a term");
    }
    // English: the same word in both vocabularies.
    for translated in [false, true] {
        let _t = override_git_terms(translated);
        assert_eq!(msg::lfs_ui::VERB_PUSH.get(), "push");
        assert_eq!(msg::autostash::REASON_PULL.get(), "the pull");
    }
    for (lang, english, noun) in [
        (Lang::Fr, "push", "envoi"),
        (Lang::Es, "push", "envío"),
        (Lang::PtBr, "push", "envio"),
        (Lang::It, "push", "invio"),
        (Lang::Ru, "push", "отправка"),
    ] {
        with_lang(lang, || {
            {
                let _t = override_git_terms(false);
                assert_eq!(msg::lfs_ui::VERB_PUSH.get(), english, "{}", lang.code());
            }
            let _t = override_git_terms(true);
            assert_eq!(msg::lfs_ui::VERB_PUSH.get(), noun, "{}", lang.code());
        });
    }
    // The article follows the noun in the translated vocabulary.
    with_lang(Lang::Es, || {
        {
            let _t = override_git_terms(false);
            assert_eq!(msg::autostash::REASON_PULL.get(), "del pull");
        }
        let _t = override_git_terms(true);
        assert_eq!(msg::autostash::REASON_PULL.get(), "de la traída");
    });
    with_lang(Lang::It, || {
        let _t = override_git_terms(true);
        assert_eq!(msg::autostash::REASON_PULL.get(), "allo scaricamento");
        assert_eq!(msg::autostash::REASON_MERGE.get(), "all'unione");
    });
    with_lang(Lang::Fr, || {
        let _t = override_git_terms(true);
        assert_eq!(msg::autostash::REASON_MERGE.get(), "la fusion");
        assert_eq!(msg::autostash::REASON_REBASE.get(), "le rebasage");
    });
}

/// Counts are composed from one plural message per number, so each noun takes
/// the language's own plural category (never "file(s)").
#[test]
fn counted_phrases_are_real_plurals_in_every_language() {
    use msg::log_draw as d;
    let tooltip = |f: u64, i: u64, l: u64| {
        d::changes_tooltip(
            d::changes_files(f),
            d::changes_insertions(i),
            d::changes_deletions(l),
        )
    };
    assert_eq!(tooltip(1, 1, 1), "1 file changed, 1 insertion, 1 deletion");
    assert_eq!(
        tooltip(3, 0, 2),
        "3 files changed, 0 insertions, 2 deletions"
    );
    with_lang(Lang::Ru, || {
        assert_eq!(d::changes_files(1), "1 изменённый файл");
        assert_eq!(d::changes_files(3), "3 изменённых файла");
        assert_eq!(d::changes_files(5), "5 изменённых файлов");
        assert_eq!(d::changes_files(21), "21 изменённый файл");
        assert_eq!(msg::settings_general::retention_unit_weeks(22), "22 недели");
    });
    with_lang(Lang::Cs, || {
        assert_eq!(msg::settings_general::retention_unit_days(1), "1 den");
        assert_eq!(msg::settings_general::retention_unit_days(2), "2 dny");
        assert_eq!(msg::settings_general::retention_unit_days(5), "5 dní");
    });
    with_lang(Lang::Fr, || {
        assert_eq!(
            tooltip(1, 2, 0),
            "1 fichier modifié, 2 insertions, 0 suppression"
        );
    });
    with_lang(Lang::Ja, || {
        assert_eq!(tooltip(2, 3, 4), "2 ファイルを変更、3 行追加、4 行削除");
    });
    assert_eq!(msg::settings_general::retention_unit_days(1), "1 day");
    assert_eq!(msg::settings_general::retention_unit_years(45), "45 years");
    with_lang(Lang::Fr, || {
        assert_eq!(msg::settings_general::retention_unit_months(1), "1 mois");
        assert_eq!(msg::settings_general::retention_unit_years(3), "3 ans");
    });
    assert_eq!(
        d::help_title_filtered(1, "x"),
        "Keybindings - filter \"x\" (1 line) - Esc clears, ? closes"
    );
    // No language keeps the "(s)" workaround.
    for &l in SHIPPED {
        with_lang(l, || {
            for text in [tooltip(2, 2, 2), d::help_title_filtered(2, "x")] {
                assert!(
                    !text.contains("(s)") && !text.contains("(es)"),
                    "{}: {text}",
                    l.code()
                );
            }
        });
    }
}
