//! The help popup's guide tabs: the user docs in a few concise lines per
//! action (what it is for, how to reach it). Tab 0 is the key bindings table
//! (`chrome::help_text`); the others are tables of one message each, written
//! `Title|text` (the title is bold, the text wrapped under it).

use crate::chrome::wrap;
use crate::th;
use gitpoulpi_i18n::{msg::help_tabs as m, Lang, M};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};

/// Where the user docs live (a tab's page is appended).
const DOCS_BASE: &str = "https://github.com/Poulpi-Lab/GitPoulpi/blob/main/docs/user/";

/// One tab: its label, the docs page behind `Ctrl+D`, its entries.
pub(crate) struct Tab {
    pub(crate) label: M,
    pub(crate) page: &'static str,
    /// Empty for the key bindings tab.
    pub(crate) entries: &'static [M],
}

pub(crate) const TABS: [Tab; 8] = [
    Tab {
        label: m::TAB_KEYS,
        page: "keybindings.md",
        entries: &[],
    },
    Tab {
        label: m::TAB_START,
        page: "interface-tour.md",
        entries: &[
            m::START_1,
            m::START_2,
            m::START_3,
            m::START_4,
            m::START_5,
            m::START_6,
        ],
    },
    Tab {
        label: m::TAB_LOG,
        page: "log-and-history.md",
        entries: &[m::LOG_1, m::LOG_2, m::LOG_3, m::LOG_4, m::LOG_5, m::LOG_6],
    },
    Tab {
        label: m::TAB_CHANGES,
        page: "staging-and-committing.md",
        entries: &[
            m::CHANGES_1,
            m::CHANGES_2,
            m::CHANGES_3,
            m::CHANGES_4,
            m::CHANGES_5,
            m::CHANGES_6,
            m::CHANGES_7,
        ],
    },
    Tab {
        label: m::TAB_REFS,
        page: "log-and-history.md#branches-tags-and-the-refs-sidebar",
        entries: &[
            m::REFS_1,
            m::REFS_2,
            m::REFS_3,
            m::REFS_4,
            m::REFS_5,
            m::REFS_6,
        ],
    },
    Tab {
        label: m::TAB_HOSTING,
        page: "hosting-integration.md",
        entries: &[
            m::HOSTING_1,
            m::HOSTING_2,
            m::HOSTING_3,
            m::HOSTING_4,
            m::HOSTING_5,
            m::HOSTING_6,
            m::HOSTING_7,
        ],
    },
    Tab {
        label: m::TAB_SETUP,
        page: "settings.md",
        entries: &[
            m::SETUP_1,
            m::SETUP_2,
            m::SETUP_3,
            m::SETUP_4,
            m::SETUP_5,
            m::SETUP_6,
        ],
    },
    Tab {
        label: m::TAB_TOOLS,
        page: "ai.md",
        entries: &[m::TOOLS_1, m::TOOLS_2, m::TOOLS_3, m::TOOLS_4, m::TOOLS_5],
    },
];

/// The docs page URL of tab `i`.
pub(crate) fn docs_url(i: usize) -> String {
    format!("{DOCS_BASE}{}", TABS[i.min(TABS.len() - 1)].page)
}

/// `Title|text` split in two (a message without a bar is all text).
fn split(entry: &str) -> (&str, &str) {
    entry.split_once('|').unwrap_or(("", entry))
}

/// Whether the entry matches `needle` (lower case) in `lang` or in English.
fn matches(entry: M, lang: Lang, needle: &str) -> bool {
    needle.is_empty()
        || entry.get_in(lang).to_lowercase().contains(needle)
        || entry.english().to_lowercase().contains(needle)
}

/// The text lines of guide tab `tab` at `width` cells, narrowed to the
/// entries matching `filter`.
pub(crate) fn lines(tab: usize, filter: &str, width: usize) -> Vec<Line<'static>> {
    let width = width.max(24);
    let needle = filter.to_lowercase();
    let lang = gitpoulpi_i18n::current();
    let title = Style::default()
        .fg(th().ui.accent)
        .add_modifier(Modifier::BOLD);
    let mut out: Vec<Line<'static>> = Vec::new();
    for entry in TABS[tab.min(TABS.len() - 1)].entries {
        if !matches(*entry, lang, &needle) {
            continue;
        }
        if !out.is_empty() {
            out.push(Line::default());
        }
        let (head, text) = split(entry.get());
        out.push(Line::from(Span::styled(
            format!(
                "  {}",
                crate::ui_kit::truncate_with_ellipsis(head, width - 2)
            ),
            title,
        )));
        for l in wrap(text, width - 4, width - 4) {
            out.push(Line::from(format!("    {l}")));
        }
    }
    out
}

/// Plain text of [`lines`], for tests.
#[cfg(test)]
pub(crate) fn text(tab: usize, filter: &str, width: usize) -> String {
    lines(tab, filter, width)
        .iter()
        .map(|l| {
            l.spans
                .iter()
                .map(|s| s.content.as_ref())
                .collect::<String>()
        })
        .collect::<Vec<_>>()
        .join("\n")
}
