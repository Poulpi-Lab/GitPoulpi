//! The "Text format of the server" row of a connected Redmine profile.
//!
//! Redmine's API does not say whether the server formats text as Textile
//! or Markdown, so the profile does (`markup`): comments are read and
//! written in it. One row that switches between the two, saved at once
//! like the rest of the page.

use super::*;
use crate::th;
use gitpoulpi_config::RedmineMarkup;
use gitpoulpi_i18n::msg::redmine_settings as rmsg;

/// The format's name: proper nouns, not translated.
fn format_name(markup: RedmineMarkup) -> &'static str {
    match markup {
        RedmineMarkup::Textile => "Textile",
        RedmineMarkup::Markdown => "Markdown",
    }
}

impl IntegrationsPage {
    /// Switches the selected Redmine profile between Textile and Markdown.
    pub(super) fn toggle_redmine_markup(&mut self, env: &mut Env) {
        let Some(i) = self.selected_profile(env.config) else {
            return;
        };
        let profile = &mut env.config.integrations[i];
        profile.markup = profile.markup.toggled();
        Self::write_config(env);
    }

    /// Paints the row and its help from row `y`; returns the next free row.
    pub(super) fn render_redmine_markup(
        &mut self,
        frame: &mut Canvas,
        body: Rect,
        mut y: u16,
        e: &Integration,
        focused: bool,
        current: Option<Item>,
    ) -> u16 {
        self.control(
            frame,
            body,
            body.x,
            y,
            rmsg::markup_row(format_name(e.markup)),
            Item::RedmineMarkup,
            current,
            focused,
            false,
        );
        y += 1;
        y += put_wrapped(
            frame,
            body,
            body.x,
            y,
            rmsg::MARKUP_HELP.get(),
            Style::default().fg(th().ui.muted),
            3,
        );
        y + 1
    }
}
