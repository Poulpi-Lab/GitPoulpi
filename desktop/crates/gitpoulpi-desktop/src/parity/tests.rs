use std::collections::BTreeSet;

use super::*;

const HEADER: &str = "# Which registry rows the desktop frontend marks done.\n\
#\n\
# Ids are the shared registry's: an action's config id (`global.quit`) or `palette.<command id>`.\n\
# Only `done` is listed; `terminal only` comes from the registry and everything else is `to do`.\n\
# The tests make this list equal to what the command dispatcher implements, so an id cannot be\n\
# added or kept by hand: implement it in `src/commands/ops.rs`, then run\n\
# `UPDATE_PARITY=1 cargo test -p gitpoulpi-desktop parity` (it rewrites this file and the table).\n\
# See README.md.\n";

fn updating() -> bool {
    std::env::var_os("UPDATE_PARITY").is_some_and(|v| !v.is_empty() && v != "0")
}

/// The notes `status.toml` already carries, by id.
fn notes() -> BTreeMap<String, String> {
    #[derive(Deserialize)]
    struct File {
        #[serde(default)]
        action: Vec<Entry>,
    }
    #[derive(Deserialize)]
    struct Entry {
        id: String,
        note: Option<String>,
    }
    let Ok(text) = std::fs::read_to_string(status_path()) else {
        return BTreeMap::new();
    };
    toml::from_str::<File>(&text)
        .map(|f| {
            f.action
                .into_iter()
                .filter_map(|e| e.note.map(|n| (e.id, n)))
                .collect()
        })
        .unwrap_or_default()
}

/// The text of `status.toml` for these done ids, keeping the notes already written.
fn status_text(done: &BTreeSet<String>, notes: &BTreeMap<String, String>) -> String {
    let mut out = String::from(HEADER);
    for id in done {
        out.push_str(&format!("\n[[action]]\nid = \"{id}\"\nstatus = \"done\"\n"));
        if let Some(note) = notes.get(id) {
            out.push_str(&format!("note = {}\n", toml::Value::String(note.clone())));
        }
    }
    out
}

/// Rewrites `status.toml` and the table when `UPDATE_PARITY` is set.
fn update_if_asked() {
    if !updating() {
        return;
    }
    let done = dispatcher_done();
    std::fs::write(status_path(), status_text(&done, &notes())).expect("write status.toml");
    let table = render(&rows_with(&done));
    std::fs::create_dir_all(table_path().parent().unwrap()).unwrap();
    std::fs::write(table_path(), table).expect("write parity.md");
}

fn diff(a: &BTreeSet<String>, b: &BTreeSet<String>) -> Vec<String> {
    a.difference(b).cloned().collect()
}

#[test]
fn the_table_on_disk_is_up_to_date() {
    update_if_asked();
    let done = status_ids().expect("status.toml");
    let expected = render(&rows_with(&done));
    let on_disk = std::fs::read_to_string(table_path()).unwrap_or_default();
    assert!(
        on_disk == expected,
        "desktop/docs/parity.md is stale: run `UPDATE_PARITY=1 cargo test -p gitpoulpi-desktop parity` \
         from desktop/ and commit the result"
    );
}

#[test]
fn every_id_in_status_toml_is_a_row_and_not_terminal_only() {
    let done = status_ids().expect("status.toml");
    let rows = rows_with(&BTreeSet::new());
    let by_id: BTreeMap<&str, &Row> = rows.iter().map(|r| (r.id.as_str(), r)).collect();
    for id in &done {
        let row = by_id
            .get(id.as_str())
            .unwrap_or_else(|| panic!("`{id}` is not a row of the registry's parity table"));
        assert!(
            !matches!(row.state, State::TerminalOnly(_)),
            "`{id}` is terminal only: it cannot be done"
        );
    }
}

#[test]
fn status_toml_is_exactly_what_the_dispatcher_implements() {
    update_if_asked();
    let marked = status_ids().expect("status.toml");
    let implemented = dispatcher_done();
    assert!(
        marked == implemented,
        "status.toml and the dispatcher disagree.\n\
         marked done but not implemented in src/commands/ops.rs: {:?}\n\
         implemented but not marked (run UPDATE_PARITY=1): {:?}",
        diff(&marked, &implemented),
        diff(&implemented, &marked),
    );
}

/// The merge criterion of the `desktop` branch: no row is left `to do`.
/// Run with `cargo test -p gitpoulpi-desktop parity_is_complete -- --ignored`.
#[test]
#[ignore = "fails until every row is done or terminal only; the merge criterion of the desktop branch"]
fn parity_is_complete() {
    let rows = rows_with(&dispatcher_done());
    let todo: Vec<&str> = rows
        .iter()
        .filter(|r| r.state == State::ToDo)
        .map(|r| r.id.as_str())
        .collect();
    assert!(
        todo.is_empty(),
        "{} rows are still to do, for instance: {:?}",
        todo.len(),
        &todo[..todo.len().min(15)]
    );
}

#[test]
fn a_terminal_only_row_is_never_to_do() {
    let rows = rows_with(&BTreeSet::new());
    let terminal: Vec<_> = rows
        .iter()
        .filter(|r| matches!(r.state, State::TerminalOnly(_)))
        .collect();
    assert!(terminal.len() >= 8, "the terminal-only settings are listed");
    assert!(terminal
        .iter()
        .all(|r| matches!(&r.state, State::TerminalOnly(why) if !why.is_empty())));
}

#[test]
fn a_row_is_done_when_its_id_is_marked() {
    let none = rows_with(&BTreeSet::new());
    assert_eq!(counts(&none).done, 0);
    let one: BTreeSet<String> = ["global.quit".to_string()].into();
    let rows = rows_with(&one);
    assert_eq!(counts(&rows).done, 1);
    assert_eq!(
        rows.iter().find(|r| r.id == "global.quit").unwrap().state,
        State::Done
    );
    assert_eq!(counts(&rows).total(), counts(&none).total());
}

#[test]
fn the_only_palette_twin_of_an_action_shares_its_row() {
    let ids: BTreeSet<String> = base_rows().into_iter().map(|r| r.id).collect();
    assert!(ids.contains("global.quit"));
    assert!(!ids.contains("palette.quit"), "merged into global.quit");
    // Several commands share the Ctrl+N prefix: each keeps its own row.
    assert!(ids.contains("palette.focus_refs"));
    assert!(ids.contains("global.nav_prefix"));
}

#[test]
fn the_markdown_has_the_summary_first_and_escapes_cells() {
    let rows = vec![
        Row {
            id: "a.b".into(),
            kind: ParityKind::Action,
            ctx: "a".into(),
            label_en: "Pipe | label".into(),
            keys: vec!["Ctrl+|".into()],
            state: State::Done,
        },
        Row {
            id: "c.d".into(),
            kind: ParityKind::Family,
            ctx: "c".into(),
            label_en: "x".into(),
            keys: vec![],
            state: State::TerminalOnly("because".into()),
        },
        Row {
            id: "e.f".into(),
            kind: ParityKind::Palette,
            ctx: "palette".into(),
            label_en: "y".into(),
            keys: vec![],
            state: State::ToDo,
        },
    ];
    let md = render(&rows);
    assert!(md.contains("**3 rows: 1 done, 1 terminal only, 1 to do.**"));
    assert!(md.contains("Pipe \\| label"));
    assert!(md.contains("terminal only: because"));
    assert!(md.find("**3 rows").unwrap() < md.find("| Id |").unwrap());
}

#[test]
fn status_toml_accepts_only_done_and_no_duplicates() {
    assert!(parse_status("[[action]]\nid = \"a\"\nstatus = \"done\"\n").is_ok());
    assert!(parse_status("[[action]]\nid = \"a\"\nstatus = \"to do\"\n").is_err());
    assert!(parse_status(
        "[[action]]\nid = \"a\"\nstatus = \"done\"\n[[action]]\nid = \"a\"\nstatus = \"done\"\n"
    )
    .is_err());
}

/// The table drops nothing the dispatcher implements: an implemented id that has no row of its own is
/// a palette command merged into the row of its key action, and that row is counted done.
#[test]
fn every_implemented_id_is_counted_in_the_table() {
    let implemented = crate::commands::implemented_ids();
    let rows: BTreeSet<String> = base_rows().into_iter().map(|r| r.id).collect();
    let done = dispatcher_done();
    let all = parity_rows();
    for id in implemented.difference(&rows) {
        let action = all
            .iter()
            .find(|r| &r.id == id)
            .and_then(|r| r.bound_action.clone())
            .unwrap_or_else(|| panic!("{id} is implemented but is in no row of the table"));
        assert!(
            done.contains(&action),
            "{id} is merged into {action}, which is not counted done"
        );
    }
    assert_eq!(done.len(), rows.intersection(&implemented).count());
}

#[test]
fn the_generated_status_file_round_trips() {
    let done: BTreeSet<String> = ["global.quit".to_string(), "global.palette".to_string()].into();
    let notes: BTreeMap<String, String> =
        [("global.quit".to_string(), "a \"note\"".to_string())].into();
    let text = status_text(&done, &notes);
    assert_eq!(parse_status(&text).unwrap(), done);
    let value: toml::Value = toml::from_str(&text).unwrap();
    let first_noted = value["action"]
        .as_array()
        .unwrap()
        .iter()
        .find(|a| a["id"].as_str() == Some("global.quit"))
        .unwrap();
    assert_eq!(first_noted["note"].as_str(), Some("a \"note\""));
}
