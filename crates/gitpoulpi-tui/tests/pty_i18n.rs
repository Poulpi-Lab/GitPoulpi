//! The display language, end to end through a pty: the real binary, the real
//! daemon, a temp config dir (never the maintainer's own).
//!
//! - `GITPOULPI_LANG=fr` draws the toolbar in French, and beats the config;
//! - Settings > General > Language switches the toolbar live (no
//!   restart), and the "Translate git terms" checkbox below it swaps
//!   `Stash` for `Remiser` and back.
//!
//! The harness pins `GITPOULPI_LANG=en` for every other suite; tests here
//! override it (an empty value is ignored, so the config decides).
//!
//! The rules are `pty_ui.rs`'s: nothing is located by a hard-coded
//! coordinate, nothing sleeps, and every wait describes the state it waits
//! for.

mod common;

use std::time::Duration;

use gitpoulpi_testkit::{FixtureRepo, FixtureRepoBuilder};
use xtask::pty::Terminal;

const STEP: Duration = Duration::from_secs(30);

fn fixture() -> FixtureRepo {
    FixtureRepoBuilder::new()
        .unwrap()
        .commit_file("a.txt", "one\n", "initial commit")
        .unwrap()
        .build()
}

fn launch(
    repo: &FixtureRepo,
    session: &str,
    lang_env: &str,
    config: &str,
) -> (Terminal, tempfile::TempDir) {
    launch_sized(repo, session, lang_env, config, 200)
}

fn launch_sized(
    repo: &FixtureRepo,
    session: &str,
    lang_env: &str,
    config: &str,
    cols: u16,
) -> (Terminal, tempfile::TempDir) {
    common::launch(repo.root())
        .session(session)
        .size(cols, 50)
        .config(config)
        // `auto` must not depend on the machine running the test.
        .env("LC_ALL", "en_US.UTF-8")
        .env("LANG", "en_US.UTF-8")
        // Windows ignores LANG/LC_ALL: pin the OS locale `auto` reads as well.
        .env("GITPOULPI_SYSTEM_LOCALE", "en-US")
        .env("GITPOULPI_LANG", lang_env)
        .ready_when("the fixture loaded", |s| s.contains("initial commit"))
        .mouse_capture()
        .start()
}

fn stored(config: &tempfile::TempDir) -> String {
    std::fs::read_to_string(config.path().join("config.toml")).unwrap_or_default()
}

/// Opens Settings on the General page: `Language` is its first row.
fn open_ui_page(term: &mut Terminal, language_label: &str) {
    let (col, row) = term
        .wait_for_text("\u{2699}", STEP)
        .unwrap_or_else(|e| panic!("{e}"));
    term.click(col, row).expect("click the gear");
    term.wait_for("Settings open", STEP, |s| {
        s.contains("Repo-specific settings") || s.contains("Param\u{e8}tres du d\u{e9}p\u{f4}t")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    // Settings reopens on the page it was left on: pick the page by name.
    let (col, row) = term
        .wait_for_text(
            if !matches!(language_label, "Langue" | "Traduire les termes git") {
                "General"
            } else {
                "G\u{e9}n\u{e9}ral"
            },
            STEP,
        )
        .unwrap_or_else(|e| panic!("{e}"));
    term.click(col + 1, row).expect("click the page");
    term.wait_for("the General page", STEP, |s| s.contains(language_label))
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
}

/// Opens the Language dropdown and picks `option`.
fn pick_language(term: &mut Terminal, current: &str, option: &str) {
    let (col, row) = common::wait_for_choice(term, current, STEP);
    term.click(col + 1, row).expect("click the language choice");
    term.wait_for("the popup listing the languages", STEP, |s| {
        s.contains("\u{2713}")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    let screen = term.screen();
    let (c, r) = screen
        .find_all(option)
        .into_iter()
        .max_by_key(|(_, r)| *r)
        .unwrap_or_else(|| panic!("{option:?} not in the popup:\n{}", term.text()));
    // A searchable popup: a double click picks (one click selects).
    term.double_click(c + 1, r)
        .expect("double-click the language");
}

fn toggle_git_terms(term: &mut Terminal, label: &str) {
    let (col, row) = term
        .wait_for_text(label, STEP)
        .unwrap_or_else(|e| panic!("{e}"));
    term.click(col, row).expect("click the checkbox row");
}

#[test]
fn gitpoulpi_lang_draws_the_toolbar_in_french_and_beats_the_config() {
    let repo = fixture();
    // The config says German; the environment says French.
    let (term, _config) = launch(&repo, "pty-i18n-env", "fr", "[ui]\nlanguage = \"de\"\n");
    term.wait_for("the French toolbar", STEP, |s| {
        s.contains("Annuler") && s.contains("Chercher") && !s.contains("Search")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
}

#[test]
fn the_language_setting_switches_the_ui_live_and_git_terms_follow_the_checkbox() {
    let repo = fixture();
    // An empty GITPOULPI_LANG is ignored, so ui.language decides (auto = en).
    let (mut term, config) = launch(&repo, "pty-i18n-live", "", "");
    term.wait_for("the English toolbar", STEP, |s| {
        s.contains("Search") && s.contains("Stash")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));

    open_ui_page(&mut term, "Translate git terms");
    pick_language(&mut term, "Auto (English)", "Fran\u{e7}ais");
    term.wait_for("Settings itself in French, live", STEP, |s| {
        s.contains("Traduire les termes git")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    assert!(
        stored(&config).contains("language = \"fr\""),
        "{}",
        stored(&config)
    );

    // The checkbox: git words go French.
    toggle_git_terms(&mut term, "Traduire les termes git");
    term.wait_for("the setting stored", STEP, |_| {
        stored(&config).contains("translate_git_terms = true")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", stored(&config)));
    term.send_keys("Escape").unwrap();
    term.wait_for("the French toolbar with French git words", STEP, |s| {
        s.contains("Annuler") && s.contains("Chercher") && s.contains("Remiser")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    assert!(!term.text().contains("Search"), "{}", term.text());

    // Back: git words off first, then English.
    open_ui_page(&mut term, "Traduire les termes git");
    toggle_git_terms(&mut term, "Traduire les termes git");
    term.wait_for("the setting stored", STEP, |_| {
        stored(&config).contains("translate_git_terms = false")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", stored(&config)));
    pick_language(&mut term, "Fran\u{e7}ais", "English");
    term.wait_for("Settings back in English", STEP, |s| {
        s.contains("Translate git terms")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.send_keys("Escape").unwrap();
    term.wait_for("the English toolbar again", STEP, |s| {
        s.contains("Search") && s.contains("Stash") && !s.contains("Chercher")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
}

/// The main window in French (I5a) at the two widths the maintainer uses: the
/// Log's column headers, the sidebar sections and the idle footer are
/// French, git words follow the setting, and no English chrome is left.
#[test]
fn the_main_window_is_french_at_100_and_180_columns() {
    let repo = fixture();
    for (cols, terms) in [(100u16, false), (180, false), (180, true)] {
        let config = if terms {
            "[ui]\ntranslate_git_terms = true\n"
        } else {
            ""
        };
        let (term, _config) = launch_sized(
            &repo,
            &format!("pty-i18n-main-{cols}-{terms}"),
            "fr",
            config,
            cols,
        );
        term.wait_for("the French main window", STEP, |s| {
            let t = s.text();
            t.contains("LOCALES") && t.contains("Message") && t.contains("aide")
        })
        .unwrap_or_else(|e| panic!("{cols} cols: {e}\n{}", term.text()));
        let text = term.text();
        // The sections follow the git vocabulary setting.
        let tags = if terms { "\u{c9}TIQUETTES" } else { "TAGS" };
        assert!(text.contains(tags), "{cols}: {text}");
        assert!(text.contains("DISTANT"), "{cols}: {text}");
        if cols >= 180 {
            assert!(text.contains("Auteur"), "{cols}: {text}");
        }
        for english in ["Author", "LOCAL ", "SUBMODULES", "? help", "Filter"] {
            assert!(
                !text.contains(english),
                "{cols} cols: {english:?} left\n{text}"
            );
        }
        // No line is wider than the terminal (a clipped or wrapped row would
        // show as the next row starting mid-word; the screen is exactly
        // `cols` wide, so check the borders still close).
        for line in text.lines() {
            assert!(
                line.chars().count() <= cols as usize,
                "{cols} cols: overflow {line:?}"
            );
        }
    }
}

/// The CJK languages end to end: `ui.language` in a temp config (the
/// environment variable left empty so the config decides) draws the chrome
/// translated, with double-width characters in the toolbar, the Log headers
/// and the sidebar, and no line wider than the terminal.
#[test]
fn a_cjk_ui_language_in_the_config_draws_translated_chrome() {
    use gitpoulpi_i18n::{msg, with_lang, Lang};
    let repo = fixture();
    for (code, lang) in [("ja", Lang::Ja), ("zh-CN", Lang::ZhCn), ("ko", Lang::Ko)] {
        let (search, undo) = with_lang(lang, || {
            (msg::toolbar::SEARCH.get(), msg::toolbar::UNDO.get())
        });
        let english = with_lang(Lang::En, || msg::toolbar::SEARCH.get());
        assert_ne!(search, english, "{code}: the toolbar is not translated");
        let (term, _config) = launch(
            &repo,
            &format!("pty-i18n-cjk-{code}"),
            "",
            &format!("[ui]\nlanguage = \"{code}\"\n"),
        );
        term.wait_for("the translated toolbar", STEP, |s| {
            // The harness paints a double-width glyph's second cell as a
            // blank: compare without blanks.
            let t = s.text().replace(' ', "");
            t.contains(&search.replace(' ', ""))
                && t.contains(&undo.replace(' ', ""))
                && !t.contains("Search")
        })
        .unwrap_or_else(|e| panic!("{code}: {e}\n{}", term.text()));
        // One char per cell: the harness paints a wide glyph's second cell
        // as a blank, so a row longer than the terminal is an overflow.
        for line in term.text().lines() {
            assert!(
                line.chars().count() <= 200,
                "{code}: a row is wider than the terminal: {line:?}"
            );
        }
    }
}
