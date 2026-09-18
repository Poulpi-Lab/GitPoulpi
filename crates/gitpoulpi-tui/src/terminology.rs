//! `ui.pr_terminology`: "Pull Request" or "Merge Request" in every label.
//!
//! The source strings all say "pull request" (GitHub's word). [`tr`] is
//! the one place that rewrites them to the effective term, applied where
//! text reaches the screen (sidebar, menus, palette, status messages), so
//! no call site has to know about the setting. The effective term is the
//! setting, or with `auto` the linked host's: GitLab says "merge request".
//!
//! Published like `live_settings::install_minimap`: written by
//! `install` (Settings, startup) and `set_host` (the hosting snapshot).
//!
//! Catalog messages do not use [`tr`]: they carry the `{$pr}` / `{$prs}` /
//! `{$pr_abbr}` host terms, and every change here is mirrored into
//! `gitpoulpi_i18n::set_merge_terms` ([`publish`]). `tr` remains only for the
//! English literals of the areas not extracted yet (palette, help, keymap
//! labels, integrations page, repo dialogs) and goes with the last of them.

use std::borrow::Cow;
use std::cell::Cell;

use gitpoulpi_config::PrTerminology;

thread_local! {
    /// Thread-local on purpose: the UI thread is the only reader, and unit
    /// tests (one thread each) cannot disturb each other.
    static STATE: Cell<(PrTerminology, bool)> = const { Cell::new((PrTerminology::Auto, false)) };
}

#[cfg(test)]
thread_local! {
    /// Tests: the per-thread i18n override replacing the process-wide
    /// switch, so parallel tests never see each other's terminology.
    static TEST_TERMS: std::cell::RefCell<Option<gitpoulpi_i18n::OverrideGuard>> =
        const { std::cell::RefCell::new(None) };
}

/// Mirrors the effective term into the catalog's host terms. Production: the
/// process-wide switch (the UI thread is its only writer; worker threads
/// that build messages read it too). Unit tests: this thread's override.
fn publish() {
    let merge = is_merge();
    #[cfg(not(test))]
    gitpoulpi_i18n::set_merge_terms(merge);
    #[cfg(test)]
    TEST_TERMS.with(|g| {
        // Drop the old guard first: a new one would record it as "previous".
        drop(g.borrow_mut().take());
        *g.borrow_mut() = Some(gitpoulpi_i18n::override_merge_terms(merge));
    });
}

pub(crate) fn install(setting: PrTerminology) {
    STATE.with(|s| s.set((setting, s.get().1)));
    publish();
}

/// The linked provider is GitLab (`auto` then says "Merge Request").
pub(crate) fn set_host(gitlab: bool) {
    STATE.with(|s| s.set((s.get().0, gitlab)));
    publish();
}

/// Whether the effective term is "merge request".
pub(crate) fn is_merge() -> bool {
    let (setting, gitlab) = STATE.with(Cell::get);
    match setting {
        PrTerminology::PullRequest => false,
        PrTerminology::MergeRequest => true,
        PrTerminology::Auto => gitlab,
    }
}

// i18n-scan: off
/// Rewrites every "pull request" in `text` to the effective term.
pub(crate) fn tr(text: &str) -> Cow<'_, str> {
    let lower = text.to_ascii_lowercase();
    if !lower.contains("pull request") && !lower.contains("pull/merge") {
        return Cow::Borrowed(text);
    }
    let merge = is_merge();
    let (word_t, word_l, word_u) = if merge {
        ("Merge", "merge", "MERGE")
    } else {
        ("Pull", "pull", "PULL")
    };
    let mut out = text.to_string();
    // "Pull/Merge Request" is the neutral spelling some labels carry.
    out = out.replace("Pull/Merge Request", &format!("{word_t} Request"));
    out = out.replace("pull/merge request", &format!("{word_l} request"));
    if merge {
        out = out
            .replace("Pull Request", "Merge Request")
            .replace("Pull request", "Merge request")
            .replace("pull request", "merge request")
            .replace("PULL REQUEST", &format!("{word_u} REQUEST"));
    }
    Cow::Owned(out)
}
// i18n-scan: on

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rewrites_every_spelling_when_merge_and_leaves_pull_alone() {
        install(PrTerminology::MergeRequest);
        assert_eq!(tr("PULL REQUESTS (3)"), "MERGE REQUESTS (3)");
        assert_eq!(tr("My Pull Requests"), "My Merge Requests");
        assert_eq!(tr("Start a pull request"), "Start a merge request");
        assert_eq!(
            tr("Open Linked Pull/Merge Request"),
            "Open Linked Merge Request"
        );
        assert_eq!(gitpoulpi_i18n::msg::hosting_ui::PR_ABBR.get(), "MR");
        install(PrTerminology::PullRequest);
        assert_eq!(tr("My Merge Requests"), "My Merge Requests");
        assert_eq!(
            tr("Open Linked Pull/Merge Request"),
            "Open Linked Pull Request"
        );
        assert_eq!(tr("Pull Requests: Search"), "Pull Requests: Search");
        install(PrTerminology::Auto);
        set_host(true);
        assert_eq!(tr("Create Pull Request"), "Create Merge Request");
        set_host(false);
        assert_eq!(tr("Create Pull Request"), "Create Pull Request");
    }

    #[test]
    fn the_setting_and_the_host_drive_the_catalog_terms() {
        use gitpoulpi_i18n::msg::hosting_ui::PULLS_TITLE;
        install(PrTerminology::Auto);
        set_host(false);
        assert_eq!(PULLS_TITLE.get(), "PULL REQUESTS");
        set_host(true);
        assert_eq!(PULLS_TITLE.get(), "MERGE REQUESTS");
        install(PrTerminology::PullRequest);
        assert_eq!(PULLS_TITLE.get(), "PULL REQUESTS");
        install(PrTerminology::MergeRequest);
        set_host(false);
        assert_eq!(PULLS_TITLE.get(), "MERGE REQUESTS");
        // Another thread (another test) never sees this thread's choice.
        std::thread::spawn(|| assert_eq!(PULLS_TITLE.get(), "PULL REQUESTS"))
            .join()
            .unwrap();
    }
}
