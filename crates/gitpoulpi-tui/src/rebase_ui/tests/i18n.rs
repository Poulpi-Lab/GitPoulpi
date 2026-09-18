//! The interactive rebase plan in French (I8b): the controls still fit at
//! 100 columns, git terms on or off, and nothing is left as a placeholder.

use super::*;
use gitpoulpi_i18n::{override_git_terms, override_lang, Lang};

#[test]
fn the_french_plan_fits_with_git_terms_on_and_off() {
    for terms in [false, true] {
        let _l = override_lang(Lang::Fr);
        let _t = override_git_terms(terms);
        let mut f = form();
        f.dropdown = Some(1);
        let (rows, hits) = paint(&f);
        let all = rows.join("\n");
        assert!(!all.contains("{$") && !all.contains("{}"), "{all}");
        assert!(all.contains("Raccourcis"), "terms={terms}\n{all}");
        assert!(all.contains("Réinitialiser"), "terms={terms}\n{all}");
        let title = if terms {
            "Rebasage interactif"
        } else {
            "Rebase interactif"
        };
        assert!(all.contains(title), "terms={terms}\n{all}");
        // Every hit target is inside the 100-column view and the three
        // footer buttons do not overlap one another.
        for (rect, _) in &hits {
            assert!(rect.x + rect.width <= 100, "{rect:?}");
        }
        let buttons: Vec<Rect> = hits
            .iter()
            .filter(|(_, p)| {
                matches!(
                    p,
                    RebasePart::Reset | RebasePart::Cancel | RebasePart::Start
                )
            })
            .map(|(r, _)| *r)
            .collect();
        assert_eq!(buttons.len(), 4, "header and footer Cancel, Reset, Start");
        for (i, a) in buttons.iter().enumerate() {
            for b in &buttons[i + 1..] {
                assert!(!a.intersects(*b), "{a:?} overlaps {b:?}");
            }
        }
        // The open dropdown lists every choice, whole.
        let wanted = if terms {
            [
                "Garder",
                "Reformuler",
                "Compresser",
                "Absorber",
                "Abandonner",
                "Modifier",
            ]
        } else {
            ["Pick", "Reword", "Squash", "Fixup", "Drop", "Edit"]
        };
        for word in wanted {
            assert!(all.contains(word), "terms={terms}: {word}\n{all}");
        }
    }
}

#[test]
fn the_action_buttons_follow_the_widest_translated_label() {
    let _l = override_lang(Lang::Fr);
    let _t = override_git_terms(true);
    assert!(RebaseForm::action_label_width() >= display_width("Abandonner"));
    drop((_l, _t));
    assert_eq!(RebaseForm::action_label_width(), 6);
}
