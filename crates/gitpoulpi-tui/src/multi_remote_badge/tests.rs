use super::*;
use gitpoulpi_core::domain::TargetRole;

pub(crate) fn target(remote: &str, relation: RemoteRelation) -> RemoteTargetState {
    RemoteTargetState {
        remote: remote.into(),
        branch: "feat".into(),
        role: TargetRole::PushTo,
        pull: false,
        tracking: Some(format!("{remote}/feat")),
        relation,
        missing: false,
    }
}

fn sample() -> Vec<RemoteTargetState> {
    vec![
        target("origin", RemoteRelation::LocalAhead(2)),
        target(
            "gitlab",
            RemoteRelation::Diverged {
                ahead: 2,
                behind: 1,
            },
        ),
        target("backup", RemoteRelation::Absent),
    ]
}

#[test]
fn the_forms_run_from_the_wide_list_to_the_compact_count() {
    let t = sample();
    assert_eq!(
        wide_form(&t),
        "origin \u{2191}2 \u{b7} gitlab \u{2191}2\u{2193}1 \u{b7} backup new"
    );
    assert_eq!(narrow_form(&t), "\u{21c9}3 \u{2191}2\u{2193}1");
    assert_eq!(compact_form(&t), "\u{21c9}3");
}

#[test]
fn in_sync_remotes_say_so_in_the_narrow_form() {
    let t = vec![
        target("origin", RemoteRelation::UpToDate),
        target("gitlab", RemoteRelation::UpToDate),
    ];
    assert_eq!(wide_form(&t), "origin = \u{b7} gitlab =");
    assert_eq!(narrow_form(&t), "\u{21c9}2 =");
}

#[test]
fn the_most_informative_form_that_fits_wins_and_none_fits_nothing() {
    let t = sample();
    let wide = display_width(&wide_form(&t));
    let narrow = display_width(&narrow_form(&t));
    let compact = display_width(&compact_form(&t));
    assert!(wide > narrow && narrow > compact);
    assert_eq!(fit(&t, wide).as_deref(), Some(wide_form(&t).as_str()));
    assert_eq!(fit(&t, wide - 1).as_deref(), Some(narrow_form(&t).as_str()));
    assert_eq!(
        fit(&t, narrow - 1).as_deref(),
        Some(compact_form(&t).as_str())
    );
    assert_eq!(fit(&t, compact - 1), None);
    assert_eq!(fit(&[], 80), None);
}

#[test]
fn the_colour_follows_the_worst_remote() {
    let ok = vec![target("a", RemoteRelation::UpToDate)];
    assert_eq!(health(&ok), Health::InSync);
    assert_eq!(color(Health::InSync), th().status.success);

    let mut some = ok.clone();
    some.push(target("b", RemoteRelation::RemoteAhead(1)));
    assert_eq!(health(&some), Health::Pending);
    assert_eq!(color(Health::Pending), th().status.warning);
    let mut fresh = ok.clone();
    fresh.push(target("b", RemoteRelation::Absent));
    assert_eq!(health(&fresh), Health::Pending);

    let mut bad = some.clone();
    bad.push(target(
        "c",
        RemoteRelation::Diverged {
            ahead: 1,
            behind: 1,
        },
    ));
    assert_eq!(health(&bad), Health::Diverged);
    assert_eq!(color(Health::Diverged), th().status.danger);
    let mut unrelated = ok;
    unrelated.push(target("b", RemoteRelation::Unrelated));
    assert_eq!(health(&unrelated), Health::Diverged);
    let mut gone = vec![target("a", RemoteRelation::UpToDate)];
    gone[0].missing = true;
    assert_eq!(health(&gone), Health::Pending);
}

#[test]
fn the_tooltip_lists_each_remote_in_words() {
    let lines = tooltip_lines("feat", &sample());
    assert_eq!(lines.len(), 1 + 3 + 1, "{lines:?}");
    assert!(lines[0].contains("feat"), "{lines:?}");
    assert!(
        lines[1].contains("origin") && lines[1].contains("2 ahead"),
        "{lines:?}"
    );
    assert!(
        lines[2].contains("gitlab") && lines[2].contains("diverged"),
        "{lines:?}"
    );
    assert!(
        lines[3].contains("backup") && lines[3].contains("no copy"),
        "{lines:?}"
    );
}
