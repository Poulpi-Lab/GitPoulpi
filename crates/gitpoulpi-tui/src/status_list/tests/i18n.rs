//! The working-changes pane in French (I5b): the toolbar, the section
//! headers and their bulk buttons fit the third pane of a 100-column
//! window in both git vocabularies, the menus hold no placeholder, and the
//! confirmation counts follow the French plural.

use super::*;
use gitpoulpi_i18n::{override_git_terms, override_lang, Lang};

/// The third pane of a 100-column window: about 40 to 52 cells.
const PANE: u16 = 52;

fn each_french(mut f: impl FnMut(bool)) {
    for terms in [false, true] {
        let _l = override_lang(Lang::Fr);
        let _t = override_git_terms(terms);
        f(terms);
    }
}

#[test]
fn the_toolbar_fits_the_pane_in_french() {
    each_french(|terms| {
        let area = Rect::new(0, 0, PANE, 1);
        let draw =
            || status_toolbar_row(area, FileListMode::Tree, Some(true), None, None, None, true);
        let english_overflow = {
            let _l = override_lang(Lang::En);
            draw().2.is_some()
        };
        let (line, hits, overflow) = draw();
        let text: String = line.spans.iter().map(|s| s.content.as_ref()).collect();
        // Never worse than English: where English fits, French fits.
        assert!(
            overflow.is_none() || english_overflow,
            "terms={terms}: overflow at {PANE}: {text:?}"
        );
        assert!(display_width(&text) <= PANE as usize, "{text:?}");
        assert!(
            text.contains("Chemins") && text.contains("Arbre"),
            "{text:?}"
        );
        assert!(text.contains("Tout replier"), "{text:?}");
        assert!(hits.iter().all(|(r, _)| r.x + r.width <= PANE));
    });
}

#[test]
fn the_section_headers_and_bulk_buttons_fit_in_french() {
    each_french(|terms| {
        for section in StatusSection::ORDER {
            let title = format!("\u{25BE} {} (12)", section.title());
            let bulk = section.bulk_label();
            assert!(!bulk.contains("{$"), "{bulk}");
            let (rect, is_more) = status_bulk_button_layout(Rect::new(0, 0, PANE, 1), &title, bulk)
                .expect("the bulk button has room");
            let english_more = {
                let _l = override_lang(Lang::En);
                let t = format!("\u{25BE} {} (12)", section.title());
                status_bulk_button_layout(Rect::new(0, 0, PANE, 1), &t, section.bulk_label())
                    .is_some_and(|(_, more)| more)
            };
            assert!(
                !is_more || english_more,
                "terms={terms}: {title:?} + {bulk:?} fall back to the dots"
            );
            assert!(rect.x >= display_width(&title).min(PANE as usize - 3) as u16);
        }
        let (title, row) = if terms {
            ("Fichiers non indexés", "Désindexer fichier")
        } else {
            ("Fichiers unstaged", "Unstage fichier")
        };
        assert_eq!(StatusSection::Unstaged.title(), title);
        assert_eq!(StatusSection::Staged.row_label(), row);
    });
}

#[test]
fn the_menus_are_translated_and_complete() {
    use crate::context_menu::Entry;
    fn labels<A>(entries: &[Entry<A>], out: &mut Vec<String>) {
        for e in entries {
            match e {
                Entry::Action { label, .. } | Entry::Disabled { label } => out.push(label.clone()),
                Entry::Submenu { label, items } => {
                    out.push(label.clone());
                    labels(items, out);
                }
                Entry::Separator => {}
            }
        }
    }
    each_french(|terms| {
        let mut all = Vec::new();
        let file = |section, conflicted, status| {
            crate::status_pane::file_context_entries(&crate::status_pane::FileMenuInput {
                section,
                status,
                conflicted,
                path: Path::new("src/main.rs"),
                lfs: vec![
                    FileContextAction::LfsTrackFile,
                    FileContextAction::LfsTrackExtension,
                    FileContextAction::LfsLock,
                ],
                binary: None,
                editor_name: "VS Code".into(),
            })
        };
        labels(&file(StatusSection::Unstaged, false, None), &mut all);
        labels(&file(StatusSection::Unstaged, true, None), &mut all);
        labels(
            &file(
                StatusSection::Unstaged,
                false,
                Some(gitpoulpi_core::domain::FileStatus::Untracked),
            ),
            &mut all,
        );
        labels(&file(StatusSection::Staged, false, None), &mut all);
        labels(
            &crate::status_pane::folder_context_entries(StatusSection::Unstaged, Path::new("src")),
            &mut all,
        );
        for l in &all {
            assert!(
                !l.is_empty() && !l.contains("{$") && !l.contains('{'),
                "{l:?}"
            );
        }
        let stage_label = if terms { "Indexer" } else { "Stage" };
        assert!(all.contains(&stage_label.to_string()), "{all:?}");
        assert!(
            all.contains(&"Ignorer «\u{a0}main.rs\u{a0}»".to_string()),
            "{all:?}"
        );
        let stash = if terms {
            "Remiser le fichier"
        } else {
            "Stash du fichier"
        };
        assert!(all.contains(&stash.to_string()), "{all:?}");
        // The English words are gone.
        for english in [
            "Stage",
            "Discard changes...",
            "Copy file path",
            "Open folder",
        ] {
            if english == "Stage" && !terms {
                continue; // git mode keeps the git word
            }
            assert!(
                !all.iter().any(|l| l == english),
                "{english} left in {all:?}"
            );
        }
    });
}

#[test]
fn the_discard_prompt_follows_the_french_plural() {
    let _l = override_lang(Lang::Fr);
    let t = tree(&[("a.txt", false, true), ("b.txt", false, true)]);
    let one = [PathBuf::from("a.txt")];
    let two = [PathBuf::from("a.txt"), PathBuf::from("b.txt")];
    assert!(discard_confirmation_prompt(&t, &one).starts_with("Abandonner 1 fichier\u{a0}?"));
    assert!(discard_confirmation_prompt(&t, &two).starts_with("Abandonner 2 fichiers\u{a0}?"));
    assert!(
        working_changes_title(1, "main", 0).contains("1 fichier modifié sur main"),
        "{}",
        working_changes_title(1, "main", 0)
    );
    assert!(working_changes_title(3, "main", 2).contains("2 EN CONFLIT"));
}
