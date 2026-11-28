//! The parity table: what the desktop app covers compared with the terminal UI, generated.
//!
//! One row per key action, palette command and generated command family of the shared registry
//! (`gitpoulpi_app::registry::parity_rows`, every feature on), each `done`, `terminal only` (with the
//! registry's reason) or `to do`. `desktop/parity/status.toml` lists the ids marked `done`; the tests
//! make that list equal to what the command dispatcher really implements, so nobody can mark a row
//! done by hand. The table is written to `desktop/docs/parity.md` (`UPDATE_PARITY=1`, see
//! `desktop/parity/README.md`).

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use gitpoulpi_app::registry::{parity_rows, ParityKind, ParityRow};
use gitpoulpi_app::surface::Surface;
use serde::Deserialize;

/// Where a row stands.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum State {
    Done,
    /// Stays in the terminal UI; the registry says why.
    TerminalOnly(String),
    ToDo,
}

/// One row of the generated table.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Row {
    pub id: String,
    pub kind: ParityKind,
    pub ctx: String,
    pub label_en: String,
    pub keys: Vec<String>,
    pub state: State,
}

/// The table's rows, before status: palette commands that are the only twin of a key action are merged
/// into that action's row (a key and its palette entry are one capability). A key action with several
/// palette twins (the focus commands behind `Ctrl+N`, the context menu commands) keeps them as rows of
/// their own, so each can be done or not on its own.
pub fn base_rows() -> Vec<ParityRow> {
    let all = parity_rows();
    let mut twins: BTreeMap<&str, usize> = BTreeMap::new();
    for r in &all {
        if let Some(a) = r.bound_action.as_deref() {
            *twins.entry(a).or_default() += 1;
        }
    }
    let merged = |r: &ParityRow| {
        r.kind == ParityKind::Palette
            && r.bound_action
                .as_deref()
                .is_some_and(|a| twins.get(a) == Some(&1))
    };
    let kept: Vec<ParityRow> = all.iter().filter(|r| !merged(r)).cloned().collect();
    kept.into_iter()
        .map(|mut r| {
            // A merged palette row can carry keys the action row lacks (a plain-text shortcut).
            if r.kind == ParityKind::Action {
                if let Some(twin) = all
                    .iter()
                    .find(|p| merged(p) && p.bound_action.as_deref() == Some(r.id.as_str()))
                {
                    for k in &twin.default_keys {
                        if !r.default_keys.contains(k) {
                            r.default_keys.push(k.clone());
                        }
                    }
                }
            }
            r
        })
        .collect()
}

/// The table's rows with their state, `done` being the ids in `done`.
pub fn rows_with(done: &BTreeSet<String>) -> Vec<Row> {
    base_rows()
        .into_iter()
        .map(|r| {
            let state = match r.surface {
                Surface::TerminalOnly(reason) => State::TerminalOnly(reason.to_string()),
                Surface::Any if done.contains(&r.id) => State::Done,
                Surface::Any => State::ToDo,
            };
            Row {
                id: r.id,
                kind: r.kind,
                ctx: r.ctx.to_string(),
                label_en: r.label_en,
                keys: r.default_keys,
                state,
            }
        })
        .collect()
}

/// The ids the dispatcher implements that are rows of the table.
pub fn dispatcher_done() -> BTreeSet<String> {
    let in_table: BTreeSet<String> = base_rows().into_iter().map(|r| r.id).collect();
    crate::commands::implemented_ids()
        .into_iter()
        .filter(|id| in_table.contains(id))
        .collect()
}

#[derive(Debug, Default, PartialEq, Eq)]
pub struct Counts {
    pub done: usize,
    pub terminal_only: usize,
    pub to_do: usize,
}

impl Counts {
    pub fn total(&self) -> usize {
        self.done + self.terminal_only + self.to_do
    }
}

pub fn counts(rows: &[Row]) -> Counts {
    let mut c = Counts::default();
    for r in rows {
        match r.state {
            State::Done => c.done += 1,
            State::TerminalOnly(_) => c.terminal_only += 1,
            State::ToDo => c.to_do += 1,
        }
    }
    c
}

/// One cell of the Markdown table: no pipe, no line break.
fn cell(text: &str) -> String {
    text.replace('|', "\\|").replace(['\n', '\r'], " ")
}

fn kind_name(kind: ParityKind) -> &'static str {
    match kind {
        ParityKind::Action => "action",
        ParityKind::Palette => "palette command",
        ParityKind::Family => "family",
    }
}

/// The Markdown of the table.
pub fn render(rows: &[Row]) -> String {
    let c = counts(rows);
    let mut out = String::new();
    out.push_str("# Parity with the terminal UI\n\n");
    out.push_str(
        "Generated from the shared registry (`gitpoulpi_app::registry::parity_rows`, every feature on) and\n\
         `desktop/parity/status.toml`. Do not edit by hand: run\n\
         `UPDATE_PARITY=1 cargo test -p gitpoulpi-desktop parity` from `desktop/` to rewrite it.\n\
         The branch merges when `parity_is_complete` passes (see `desktop/parity/README.md`).\n\n",
    );
    out.push_str(&format!(
        "**{} rows: {} done, {} terminal only, {} to do.**\n\n",
        c.total(),
        c.done,
        c.terminal_only,
        c.to_do
    ));
    out.push_str("| Kind | Rows | Done | Terminal only | To do |\n|---|---:|---:|---:|---:|\n");
    for kind in [ParityKind::Action, ParityKind::Palette, ParityKind::Family] {
        let of_kind: Vec<Row> = rows.iter().filter(|r| r.kind == kind).cloned().collect();
        let k = counts(&of_kind);
        out.push_str(&format!(
            "| {} | {} | {} | {} | {} |\n",
            kind_name(kind),
            k.total(),
            k.done,
            k.terminal_only,
            k.to_do
        ));
    }
    out.push_str(
        "\nA palette command that is the only twin of a key action shares that action's row.\n\n",
    );
    out.push_str("| Id | Context | Label | Default keys | Status |\n|---|---|---|---|---|\n");
    for r in rows {
        let status = match &r.state {
            State::Done => "done".to_string(),
            State::TerminalOnly(reason) => format!("terminal only: {reason}"),
            State::ToDo => "to do".to_string(),
        };
        out.push_str(&format!(
            "| `{}` | {} | {} | {} | {} |\n",
            cell(&r.id),
            cell(&r.ctx),
            cell(&r.label_en),
            cell(&r.keys.join(" / ")),
            cell(&status)
        ));
    }
    out
}

/// `desktop/`, whatever the working directory of the test.
pub fn desktop_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

pub fn status_path() -> PathBuf {
    desktop_dir().join("parity/status.toml")
}

pub fn table_path() -> PathBuf {
    desktop_dir().join("docs/parity.md")
}

#[derive(Deserialize)]
struct StatusFile {
    #[serde(default)]
    action: Vec<StatusEntry>,
}

#[derive(Deserialize)]
struct StatusEntry {
    id: String,
    status: String,
    #[allow(dead_code)]
    note: Option<String>,
}

/// The ids `status.toml` marks done. An entry with another status, or a repeated id, is an error.
pub fn parse_status(text: &str) -> Result<BTreeSet<String>, String> {
    let file: StatusFile = toml::from_str(text).map_err(|e| e.to_string())?;
    let mut ids = BTreeSet::new();
    for e in file.action {
        if e.status != "done" {
            return Err(format!(
                "{}: status `{}` is not listed here: only `done` rows are (the rest is derived)",
                e.id, e.status
            ));
        }
        if !ids.insert(e.id.clone()) {
            return Err(format!("{} is listed twice", e.id));
        }
    }
    Ok(ids)
}

/// The ids marked done in `desktop/parity/status.toml`.
pub fn status_ids() -> Result<BTreeSet<String>, String> {
    let text = std::fs::read_to_string(status_path()).map_err(|e| e.to_string())?;
    parse_status(&text)
}

#[cfg(test)]
mod tests;
