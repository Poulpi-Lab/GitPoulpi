//! The rule of the Settings search: which rows a query finds, and how well.
//!
//! A frontend lists its searchable rows (a label, the page title and the help
//! of each, in the interface language and in English) and asks [`tier`] about
//! every one. Every whitespace-separated word of the query has to occur
//! (case insensitively) in the label, the page title or the help, each in the
//! interface language or in English: the English words still find a row while
//! the interface is in another language.

/// The words of a query, lower case; empty when it holds none.
pub fn query_words(query: &str) -> Vec<String> {
    query.split_whitespace().map(str::to_lowercase).collect()
}

/// The texts of one searchable row: what is shown and its English twin.
#[derive(Debug, Clone, Copy)]
pub struct Texts<'a> {
    pub label: &'a str,
    pub label_en: &'a str,
    pub page: &'a str,
    pub page_en: &'a str,
    pub help: &'a str,
    pub help_en: &'a str,
}

/// How well `words` (from [`query_words`]) match a row: `0` every word is in
/// the label, `1` in the label or the page title, `2` (a help-only match) in
/// the label, the page title or the help; `None` for a row that does not
/// match. Lower is better; equal tiers keep the order of the rows.
pub fn tier(words: &[String], texts: Texts) -> Option<u8> {
    let label = format!("{} {}", texts.label, texts.label_en).to_lowercase();
    let page = format!("{} {}", texts.page, texts.page_en).to_lowercase();
    let help = format!("{} {}", texts.help, texts.help_en).to_lowercase();
    if words.iter().all(|w| label.contains(w)) {
        Some(0)
    } else if words.iter().all(|w| label.contains(w) || page.contains(w)) {
        Some(1)
    } else if words
        .iter()
        .all(|w| label.contains(w) || page.contains(w) || help.contains(w))
    {
        Some(2)
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn texts<'a>(label: &'a str, page: &'a str, help: &'a str) -> Texts<'a> {
        Texts {
            label,
            label_en: "",
            page,
            page_en: "",
            help,
            help_en: "",
        }
    }

    #[test]
    fn a_label_match_beats_a_page_match_beats_a_help_match() {
        let words = query_words("Auto Fetch");
        assert_eq!(tier(&words, texts("Prune", "Sync", "")), None);
        assert_eq!(
            tier(&words, texts("Auto-fetch interval", "Sync", "")),
            Some(0)
        );
        assert_eq!(tier(&words, texts("Auto fetch", "Sync", "")), Some(0));
        assert_eq!(tier(&words, texts("Fetch", "Auto Sync", "")), Some(1));
        assert_eq!(tier(&words, texts("Fetch", "Sync", "runs auto")), Some(2));
    }

    #[test]
    fn the_english_words_find_a_row_in_another_language() {
        let words = query_words("minimap");
        let row = Texts {
            label: "Minikarte",
            label_en: "Minimap",
            page: "",
            page_en: "",
            help: "",
            help_en: "",
        };
        assert_eq!(tier(&words, row), Some(0));
    }

    #[test]
    fn an_empty_query_has_no_words() {
        assert!(query_words("   ").is_empty());
    }
}
