//! `Flow: Start <Type>...` rows: one palette command per topic type of the
//! repository's workflow, generated from the report (never listed by hand),
//! beside the fixed `Flow: ...` catalogue entries.
//!
//! The rows are `&'static PaletteEntry` like the catalogue's, so their text
//! is leaked: once per (language epoch, set of type labels), which is bounded
//! by language switches and edits of the workflow.

use crate::flow::FlowCmd;
use gitpoulpi_app::palette::{PaletteCommand, PaletteEntry, Surface};
use gitpoulpi_i18n::msg::flow as flow_msg;
use std::sync::Mutex;

type Key = (u64, u8, bool, Vec<String>);

static MODEL: Mutex<Option<(Key, &'static [PaletteEntry])>> = Mutex::new(None);

fn labels() -> Vec<String> {
    crate::flow::view::configured()
        .map(|v| {
            v.topics()
                .iter()
                .map(|t| t.display_label().to_string())
                .collect()
        })
        .unwrap_or_default()
}

fn leak(text: String) -> &'static str {
    Box::leak(text.into_boxed_str())
}

/// A hash of what the entries depend on besides the language: the haystack
/// cache of `matching` keys on it, so editing the types refreshes the rows.
pub(crate) fn signature() -> u64 {
    use std::hash::{Hash, Hasher};
    let mut h = std::collections::hash_map::DefaultHasher::new();
    labels().hash(&mut h);
    h.finish()
}

/// The `Flow: Start <Type>...` entries of the current workflow, in the
/// current language (none without a workflow).
pub(crate) fn entries() -> &'static [PaletteEntry] {
    let labels = labels();
    if labels.is_empty() {
        return &[];
    }
    let key: Key = (
        gitpoulpi_i18n::epoch(),
        gitpoulpi_i18n::current() as u8,
        gitpoulpi_i18n::git_terms_translated(),
        labels.clone(),
    );
    let mut slot = MODEL.lock().unwrap_or_else(|e| e.into_inner());
    if let Some((k, built)) = slot.as_ref() {
        if *k == key {
            return built;
        }
    }
    let built: Vec<PaletteEntry> = labels
        .iter()
        .enumerate()
        .flat_map(|(i, label)| {
            let Ok(i) = u8::try_from(i) else {
                return Vec::new();
            };
            let english = |f: fn(&str) -> String| {
                gitpoulpi_i18n::with_lang(gitpoulpi_i18n::Lang::En, || f(label))
            };
            vec![
                PaletteEntry {
                    label: flow_msg::PALETTE_START,
                    shortcut: "",
                    hint: None,
                    command: PaletteCommand::Flow(FlowCmd::Start(i)),
                    requires: None,
                    text: Some(leak(flow_msg::palette_start_type(label))),
                    text_en: Some(leak(english(|l| flow_msg::palette_start_type(l)))),
                    surface: Surface::Any,
                },
                PaletteEntry {
                    label: flow_msg::ISSUE_PALETTE_NEW,
                    shortcut: "",
                    hint: None,
                    command: PaletteCommand::Flow(FlowCmd::NewIssue(i)),
                    requires: None,
                    text: Some(leak(flow_msg::issue_palette_new_type(label))),
                    text_en: Some(leak(english(|l| flow_msg::issue_palette_new_type(l)))),
                    surface: Surface::Any,
                },
            ]
        })
        .collect();
    let built: &'static [PaletteEntry] = Box::leak(built.into_boxed_slice());
    *slot = Some((key, built));
    built
}
