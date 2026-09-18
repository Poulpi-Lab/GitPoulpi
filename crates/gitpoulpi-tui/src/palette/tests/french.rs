//! The palette in French: labels, reasons, widths, bilingual matching and
//! the cost of a keystroke.

use super::*;
use crate::ui_kit::display_width;
use gitpoulpi_i18n::{msg, override_git_terms, override_lang, Lang};

fn fr_labels() -> Vec<String> {
    PALETTE_COMMANDS
        .iter()
        .map(|e| e.display_label().replace('\u{a0}', " "))
        .collect()
}

/// The popup is 62 columns of a 100-column window; its rows give the label
/// what the longest shortcut leaves (about 44). A longer label is cut with an
/// ellipsis, as English ones are; the bulk must still show in full.
#[test]
fn french_labels_fit_the_hundred_column_popup() {
    let en_fits = PALETTE_COMMANDS
        .iter()
        .filter(|e| display_width(&e.display_label()) <= 44)
        .count();
    let _l = override_lang(Lang::Fr);
    let widths: Vec<usize> = PALETTE_COMMANDS
        .iter()
        .map(|e| display_width(&e.display_label()))
        .collect();
    for e in PALETTE_COMMANDS {
        let l = e.display_label();
        assert!(l.contains("\u{a0}: "), "no category: {l}");
    }
    let fits = widths.iter().filter(|w| **w <= 44).count();
    // French runs longer than English; most labels still show in full.
    assert!(
        fits * 100 >= en_fits * 65,
        "only {fits} French labels fit 44 columns ({en_fits} English)"
    );
    assert!(widths.iter().all(|w| *w <= 90), "{widths:?}");
    // The short, common commands show in full.
    for (terms, wants) in [
        (
            false,
            [
                "Conflits : Ouvrir l'éditeur de merge",
                "Git : Push",
                "Git : Pull",
                "Merge : Conflit suivant",
            ],
        ),
        (
            true,
            [
                "Conflits : Ouvrir l'éditeur de fusion",
                "Git : Pousser",
                "Git : Tirer",
                "Fusion : Conflit suivant",
            ],
        ),
    ] {
        let _t = override_git_terms(terms);
        for want in wants {
            assert!(
                fr_labels().iter().any(|l| l == want),
                "missing {want} (translate git terms: {terms})"
            );
        }
    }
}

#[test]
fn no_french_label_is_left_in_english() {
    let _l = override_lang(Lang::Fr);
    let mut same = Vec::new();
    for e in PALETTE_COMMANDS {
        let fr = e.label.get();
        let en = e.label.english();
        // Allowed to match: single-word names such as "Diff : Blame".
        if fr == en {
            same.push(en);
        }
    }
    assert!(same.is_empty(), "untranslated: {same:#?}");
}

fn first(query: &str) -> Option<PaletteCommand> {
    palette_matches(query, PaletteContext::default())
        .first()
        .map(|m| m.entry.command)
}

/// A query matches the label in the current language or in English.
#[test]
fn french_and_english_queries_both_find_the_merge_editor() {
    let _l = override_lang(Lang::Fr);
    assert_eq!(
        first("ouvrir l'éditeur"),
        Some(PaletteCommand::OpenMergeEditor)
    );
    assert_eq!(
        first("open merge editor"),
        Some(PaletteCommand::OpenMergeEditor)
    );
    // Case-insensitive, and the category-less body still wins.
    assert_eq!(
        first("OUVRIR L'ÉDITEUR"),
        Some(PaletteCommand::OpenMergeEditor)
    );
    // English keeps working in English, and French does not leak into it.
    drop(_l);
    assert_eq!(
        first("open merge editor"),
        Some(PaletteCommand::OpenMergeEditor)
    );
    assert!(palette_matches("ouvrir l'éditeur", PaletteContext::default()).is_empty());
}

#[test]
fn a_language_switch_rebuilds_the_search_text() {
    assert_eq!(first("ouvrir l'éditeur de fusion"), None);
    {
        let _l = override_lang(Lang::Fr);
        {
            let _t = override_git_terms(true);
            assert_eq!(
                first("ouvrir l'éditeur de fusion"),
                Some(PaletteCommand::OpenMergeEditor)
            );
        }
        let _t = override_git_terms(false);
        assert_eq!(
            first("ouvrir l'éditeur de merge"),
            Some(PaletteCommand::OpenMergeEditor)
        );
    }
    assert_eq!(first("ouvrir l'éditeur de fusion"), None);
}

/// Git words follow the "Translate git terms" setting.
#[test]
fn git_words_follow_the_vocabulary_setting() {
    let _l = override_lang(Lang::Fr);
    let find = |cmd: PaletteCommand| {
        PALETTE_COMMANDS
            .iter()
            .find(|e| e.command == cmd)
            .unwrap()
            .label
            .get()
    };
    assert_eq!(find(PaletteCommand::Push), "Git\u{a0}: Push");
    let _g = override_git_terms(true);
    assert_eq!(find(PaletteCommand::Push), "Git\u{a0}: Pousser");
}

#[test]
fn reasons_are_translated() {
    let _l = override_lang(Lang::Fr);
    let ctx = PaletteContext::default();
    let reason =
        unavailable_reason(PaletteCommand::CheckoutSelectedRef, ctx).expect("nothing selected");
    assert_eq!(
        reason,
        "sélectionnez d'abord une branche ou un tag dans le volet Refs"
    );
    let home = PaletteContext {
        in_repository_management: true,
        ..ctx
    };
    let grayed = palette_matches("", home)
        .into_iter()
        .find(|m| m.entry.command == PaletteCommand::FocusRefs)
        .unwrap();
    assert_eq!(grayed.unavailable, Some("nécessite un dépôt ouvert"));
    assert_eq!(
        msg::palette::NO_MATCH.get(),
        "(aucune commande ne correspond)"
    );
    assert!(msg::palette::title(3, 300).starts_with("Palette de commandes"));
    // Needs-repository detection reads the English label, so it is
    // language independent.
    assert!(needs_repository(PALETTE_COMMANDS[0].label.english()));
}

/// CPU time of this thread in microseconds: unlike the wall clock it does
/// not grow when the machine is oversubscribed and the thread waits for a
/// core (a wall-clock budget failed 62 runs of 80 copies on 20 cores).
/// Windows has no cheap equivalent here, so it falls back to wall time.
#[cfg(unix)]
fn thread_cpu_micros() -> u128 {
    let mut ts = libc::timespec {
        tv_sec: 0,
        tv_nsec: 0,
    };
    // SAFETY: `ts` is a valid out-pointer for the duration of the call.
    let ok = unsafe { libc::clock_gettime(libc::CLOCK_THREAD_CPUTIME_ID, &mut ts) } == 0;
    assert!(ok, "clock_gettime(CLOCK_THREAD_CPUTIME_ID) failed");
    ts.tv_sec as u128 * 1_000_000 + ts.tv_nsec as u128 / 1_000
}

#[cfg(not(unix))]
fn thread_cpu_micros() -> u128 {
    use std::sync::OnceLock;
    use std::time::Instant;
    static START: OnceLock<Instant> = OnceLock::new();
    START.get_or_init(Instant::now).elapsed().as_micros()
}

/// The ranking cache is per language epoch: a keystroke allocates only
/// the result rows, never the 300 lower-cased labels.
#[test]
fn matching_a_keystroke_over_every_entry_stays_cheap() {
    let ctx = PaletteContext {
        ai_enabled: true,
        ..PaletteContext::default()
    };
    for lang in [Lang::En, Lang::Fr] {
        let _l = override_lang(lang);
        // Warm the epoch's cache.
        let _ = palette_matches("", ctx);
        // The fastest of many single calls per query, in CPU time: a loaded
        // machine only adds time, and the minimum is the cost of the work.
        let mut found = 0;
        let mut total = 0u128;
        let queries = ["g", "gi", "git", "git p", "merge", "fusion", "xyzzy", ""];
        for q in queries {
            let mut fastest = u128::MAX;
            for _ in 0..200 {
                let t = thread_cpu_micros();
                found += palette_matches(q, ctx).len();
                fastest = fastest.min(thread_cpu_micros() - t);
            }
            total += fastest;
        }
        let per = total / queries.len() as u128;
        assert!(found > 0);
        // Generous: debug build on a loaded machine. Rebuilding every
        // lower-cased label per keystroke is several times this.
        assert!(per < 5_000, "{per} us per keystroke in {lang:?}");
    }
}
