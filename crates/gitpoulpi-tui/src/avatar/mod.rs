//! Author avatars (`ui.avatars`: graph / author / off).
//!
//! Capability detection, cell metrics and the iTerm2 encoder are shared with
//! [`crate::graph`] (see `crate::terminal_image`); everything specific to
//! *this* feature lives here.
//!
//! ## The pieces, and why each is its own file
//!
//! - [`gravatar`] -- builds the URL. Pure, no I/O; see its module header
//!   for what was verified about the Gravatar API.
//! - [`github`] -- GitHub as a source: noreply addresses, github.com
//!   remotes, the commits API's `author.avatar_url`. Pure, no I/O.
//! - [`resolve`] -- which sources are tried in which order, and what is
//!   remembered on disk about each answer; the network is an injected
//!   `Fetch`, so all of it is tested offline.
//! - [`initials`] -- the fallback: a colored initials badge derived
//!   deterministically from the author's name and email, no network. It
//!   renders when avatars are off, when the terminal cannot draw images, or
//!   before a fetch lands.
//! - [`cache`] -- the on-disk cache in front of the network, keyed by
//!   Gravatar hash, with hit/negative/expiry semantics.
//! - [`crate::image_decode`] -- turns fetched bytes into a [`crate::terminal_image::Bitmap`]
//!   sized for the caller's cell box.
//! - [`service`] -- the background worker: fetch, decode, cache, and the
//!   non-blocking API the render loop polls.
//!
//! ## Placement: Log row and commit-detail pane
//!
//! The avatar sits directly beside the commit's author
//! name in both the Log row and the commit-detail header, not at the graph
//! node (which carries the lane/merge shape). On a text terminal the Log
//! shows one cell: **one** letter of the initials badge, since a two-column
//! badge would not fit at the Author column's floor width.
//!
//! ## What is wired
//!
//! The initials-badge fallback ([`badge`]) is wired into the Log row and the
//! commit-detail pane, gated on `ui.show_avatars`; it needs no capability
//! detection and is correct on every terminal.
//!
//! The pixel path ([`service::AvatarService`], [`cache`], [`crate::image_decode`]) feeds
//! two pictures: the commit-detail pane's one avatar ([`detail_image`]) and
//! one per visible Log row ([`log_images`]): a round picture in the Author
//! column's two leading cells ([`LogAvatarMode::Picture`]), with the
//! two-letter initials standing in until it is fetched. A text terminal
//! keeps the one-cell glyph ([`LogAvatarMode::Glyph`]). `log_images`'
//! header explains why it is one picture per row rather than one strip for
//! the column.

mod bitbucket;
mod cache;
mod detail_image;
mod github;
mod gitlab;
mod gravatar;
mod identicon;
mod initials;
mod log_images;
mod resolve;
mod service;

use gitpoulpi_config::{
    AvatarFallback, AvatarPlacement, AvatarSources, Integration, IntegrationKind,
};
use ratatui::style::Color;

use crate::terminal_image::Capability;

pub(crate) use bitbucket::pick_repo as pick_bitbucket_repo;
pub(crate) use detail_image::{detail_size, DetailAvatarImage};
pub(crate) use github::pick_repo as pick_github_repo;
pub(crate) use gitlab::pick_host as pick_gitlab_host;
pub(crate) use log_images::{
    card_avatar_mode, log_avatar_mode, slot_area, AvatarSlot, LogAvatarImages, LogAvatarMode,
    PictureKey, RowTint, SLOT_COLS,
};
pub(crate) use resolve::TokenProvider;
pub(crate) use service::{AvatarQuery, AvatarService, AvatarStatus};

/// Whether *anything* about avatars should be attempted at all (network,
/// cache, decode). `false` short-circuits before a byte is hashed, which
/// matters because [`gravatar::email_hash`] is the privacy-relevant step: a
/// reviewer checking "is a hash ever computed when the user did not opt in"
/// should read this predicate.
pub(crate) fn enabled(placement: AvatarPlacement) -> bool {
    placement != AvatarPlacement::Off
}

/// Whether the *pixel* path should be attempted for this author, as opposed
/// to going straight to the initials badge. A pure function of the setting
/// and the terminal's capability, separate from `enabled` so a test can
/// drive every `(placement, draws_images)` combination without a terminal,
/// a service, or a network. Both `graph` and `author` show the picture in
/// the commit-detail pane.
pub(crate) fn should_fetch_images(placement: AvatarPlacement, capability: Capability) -> bool {
    enabled(placement) && capability.draws_images()
}

/// Whether the Graph column draws each commit's avatar inside its circle
/// (`ui.avatars = "graph"`): only when the graph itself is pixels.
pub(crate) fn in_graph(placement: AvatarPlacement, capability: Capability) -> bool {
    placement == AvatarPlacement::Graph && capability.draws_graph()
}

/// The avatar settings besides `ui.avatars`, re-read when Preferences
/// closes: the source order, the fallback, and the integrations hosting
/// lookups may use.
#[derive(Debug, Clone, Default)]
pub(crate) struct LiveSettings {
    pub(crate) sources: AvatarSources,
    pub(crate) fallback: AvatarFallback,
    pub(crate) integrations: Vec<Integration>,
    /// Set when the settings or the repository's remotes changed and
    /// [`LiveSettings::apply`] is owed.
    pub(crate) dirty: bool,
}

impl LiveSettings {
    pub(crate) fn from_config(config: &gitpoulpi_config::Config) -> LiveSettings {
        LiveSettings {
            sources: config.ui.avatar_sources,
            fallback: config.ui.avatar_fallback,
            integrations: config.integrations.clone(),
            dirty: true,
        }
    }

    /// Pushes everything into the service; `remotes` are `(name, fetch
    /// url)` of the window's repository. Cheap: call when any of it
    /// changed.
    pub(crate) fn apply<'a>(
        &mut self,
        service: &mut AvatarService,
        remotes: impl IntoIterator<Item = (&'a str, &'a str)>,
    ) {
        self.dirty = false;
        service.set_sources(self.sources);
        service.set_fallback(self.fallback);
        let remotes: Vec<(&str, &str)> = remotes.into_iter().collect();
        service.set_hosting(
            pick_gitlab_host(remotes.iter().copied(), &self.integrations)
                .filter(|_| crate::build_features::GITLAB),
            crate::build_features::GITHUB && github_connected(&self.integrations),
        );
        service.set_bitbucket(
            pick_bitbucket_repo(remotes.iter().copied(), &self.integrations)
                .filter(|_| crate::build_features::BITBUCKET),
        );
    }
}

/// Where the GitHub API token comes from, for the avatar worker: the
/// first `[[integrations]]` entry of kind `github` on github.com, its
/// `GITPOULPI_TOKEN_<ID>` environment variable first, then the token the
/// Preferences screen stored under its `token_ref`. `None` (the
/// unauthenticated 60-an-hour limit, public repositories only) when no
/// such integration exists. Evaluated lazily on the worker thread: the
/// store is `git credential fill`, a subprocess.
///
/// The environment-variable rule is `gitpoulpi_hosting::token_env_var`'s,
/// restated because this crate does not depend on `gitpoulpi-hosting`.
pub(crate) fn github_token_provider(integrations: &[Integration]) -> TokenProvider {
    let integration = integrations
        .iter()
        .find(|i| i.kind == IntegrationKind::Github && is_github_dot_com(&i.url))
        .cloned();
    Box::new(move || token_of(&integration?))
}

/// The token of one integration: its `GITPOULPI_TOKEN_<ID>` environment
/// variable, else the one stored under its `token_ref`.
pub(super) fn token_of(integration: &Integration) -> Option<String> {
    use gitpoulpi_secrets::TokenStore;
    let var: String = integration
        .id
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() {
                c.to_ascii_uppercase()
            } else {
                '_'
            }
        })
        .collect();
    if let Some(token) = std::env::var(format!("GITPOULPI_TOKEN_{var}"))
        .ok()
        .filter(|t| !t.trim().is_empty())
    {
        return Some(token);
    }
    gitpoulpi_secrets::CredentialHelperTokens::new()
        .load(&integration.token_ref)
        .ok()
        .flatten()
}

/// Whether an enabled github.com integration is connected (the avatar
/// worker may then use its token for the user search).
pub(crate) fn github_connected(integrations: &[Integration]) -> bool {
    integrations
        .iter()
        .any(|i| i.kind == IntegrationKind::Github && i.enabled && is_github_dot_com(&i.url))
}

/// The GitLab token for a host (`https://host`), from the enabled
/// `gitlab` integration whose URL is that host. Never returns another
/// host's token.
pub(crate) fn gitlab_token_provider(integrations: &[Integration]) -> resolve::GitLabTokens {
    let integrations: Vec<Integration> = integrations
        .iter()
        .filter(|i| gitlab::is_gitlab(i))
        .cloned()
        .collect();
    Box::new(move |base| {
        let integration = integrations
            .iter()
            .find(|i| gitlab::base_of(&i.url).eq_ignore_ascii_case(base))?;
        token_of(integration)
    })
}

fn is_github_dot_com(url: &str) -> bool {
    let rest = url.split_once("://").map_or(url, |(_, rest)| rest);
    let host = rest.split(['/', ':']).next().unwrap_or("");
    matches!(
        host.to_ascii_lowercase().as_str(),
        "github.com" | "www.github.com" | "api.github.com"
    )
}

/// The initials badge for one author: text plus a deterministic color.
/// Never fails, never does I/O - the thing every other representation in
/// this module falls back to.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Badge {
    pub(crate) initials: String,
    pub(crate) color: Color,
}

pub(crate) fn badge(author_name: &str, author_email: &str) -> Badge {
    Badge {
        initials: initials::initials(author_name),
        color: initials::color_for(author_email),
    }
}

/// The text standing in for an author's picture, `cols` cells wide, and
/// its color: the identicon as quadrant blocks, or the initials badge
/// (`ui.avatar_fallback`). One cell is the Log's glyph on a text
/// terminal, [`SLOT_COLS`] its picture slot before the picture arrives.
pub(crate) fn fallback_text(
    fallback: AvatarFallback,
    author_name: &str,
    author_email: &str,
    cols: usize,
) -> (String, Color) {
    // One cell is the text terminal's glyph (no picture can ever replace
    // it): an identicon squeezed into one cell is blank for some authors,
    // so the author is shown by the initials badge whatever the setting.
    let fallback = if cols <= 1 {
        AvatarFallback::Initials
    } else {
        fallback
    };
    match fallback {
        AvatarFallback::Identicon => (
            identicon::blocks(author_email, cols),
            identicon::color(author_email),
        ),
        AvatarFallback::Initials => {
            let badge = badge(author_name, author_email);
            let text = if cols <= 1 {
                badge_glyph(&badge).to_string()
            } else {
                badge_slot_text(&badge)
            };
            (text, badge.color)
        }
    }
}

/// The single character drawn in a one-cell-wide badge (see this module's
/// header on why the Log's avatar column is one cell): the first
/// character of the two-letter initials, which for the common two-word
/// case is the author's own first-name initial - the more identifying of
/// the two in a list sorted by nothing to do with surname.
pub(crate) fn badge_glyph(badge: &Badge) -> &str {
    // `initials` always returns at least one character, but a byte-index
    // slice would be wrong for a non-ASCII first letter; `chars()` takes
    // "the first character" correctly.
    match badge.initials.chars().next() {
        Some(_) => {
            let end = badge
                .initials
                .char_indices()
                .nth(1)
                .map(|(i, _)| i)
                .unwrap_or(badge.initials.len());
            &badge.initials[..end]
        }
        None => "?",
    }
}

/// The initials as they fill the Log's [`SLOT_COLS`]-wide picture slot
/// while the picture itself is not there (not fetched yet, no Gravatar,
/// the fetch failed): both letters, cut or space-padded to exactly
/// `SLOT_COLS` terminal columns, so the author's name after it starts in
/// the same column whether the row shows the picture or the letters.
pub(crate) fn badge_slot_text(badge: &Badge) -> String {
    let slot = usize::from(SLOT_COLS);
    let mut out = String::new();
    let mut used = 0;
    for ch in badge.initials.chars() {
        let w = crate::ui_kit::display_width(ch.encode_utf8(&mut [0; 4]));
        if used + w > slot {
            break;
        }
        out.push(ch);
        used += w;
    }
    out.push_str(&" ".repeat(slot - used));
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::terminal_image::{CellSize, Protocol};

    /// Red if `badge.initials` is returned unpadded: the one-letter case
    /// starts the name a column early.
    #[test]
    fn the_slot_text_is_always_exactly_the_slot_wide() {
        for (name, email) in [
            ("Ada Lovelace", "ada@example.com"),
            ("", ""),
            ("Cher", "cher@example.com"),
            ("山田 太郎", "yamada@example.com"),
        ] {
            let text = badge_slot_text(&badge(name, email));
            assert_eq!(
                crate::ui_kit::display_width(&text),
                usize::from(SLOT_COLS),
                "{name:?} -> {text:?}"
            );
        }
        assert_eq!(badge_slot_text(&badge("Ada Lovelace", "a@x")), "AL");
    }

    fn capability(protocol: Protocol) -> Capability {
        Capability {
            protocol,
            cell: CellSize::FALLBACK,
        }
    }

    #[test]
    fn images_are_never_fetched_when_the_setting_is_off() {
        assert!(!should_fetch_images(
            AvatarPlacement::Off,
            capability(Protocol::Iterm2)
        ));
    }

    #[test]
    fn images_are_fetched_when_on_and_the_terminal_draws_them() {
        for placement in [AvatarPlacement::Graph, AvatarPlacement::Author] {
            assert!(should_fetch_images(placement, capability(Protocol::Iterm2)));
        }
    }

    #[test]
    fn only_graph_on_an_image_terminal_puts_avatars_in_the_graph() {
        assert!(in_graph(
            AvatarPlacement::Graph,
            capability(Protocol::Iterm2)
        ));
        assert!(!in_graph(
            AvatarPlacement::Graph,
            capability(Protocol::None)
        ));
        assert!(!in_graph(
            AvatarPlacement::Author,
            capability(Protocol::Iterm2)
        ));
        assert!(!in_graph(
            AvatarPlacement::Off,
            capability(Protocol::Iterm2)
        ));
    }

    #[test]
    fn sixel_puts_avatars_in_the_graph_like_the_other_picture_protocols() {
        let sixel = capability(Protocol::Sixel);
        assert!(in_graph(AvatarPlacement::Graph, sixel));
        assert!(should_fetch_images(AvatarPlacement::Graph, sixel));
        assert_eq!(
            log_avatar_mode(AvatarPlacement::Graph, sixel),
            LogAvatarMode::Off
        );
        assert_eq!(
            log_avatar_mode(AvatarPlacement::Author, sixel),
            LogAvatarMode::Picture
        );
        assert_eq!(
            log_avatar_mode(AvatarPlacement::Graph, capability(Protocol::Iterm2)),
            LogAvatarMode::Off
        );
        assert_eq!(
            log_avatar_mode(AvatarPlacement::Author, capability(Protocol::None)),
            LogAvatarMode::Glyph
        );
    }

    /// Only a github.com integration's token is ever handed to the
    /// avatar worker - a GitLab or GitHub Enterprise token must never
    /// reach api.github.com.
    #[test]
    fn the_token_comes_only_from_a_github_dot_com_integration() {
        let none = github_token_provider(&[
            Integration::new("gl", IntegrationKind::Gitlab, "https://gitlab.com"),
            Integration::new("ghe", IntegrationKind::Github, "https://github.example.com"),
        ]);
        assert_eq!(none(), None);
        assert!(is_github_dot_com("https://github.com"));
        assert!(is_github_dot_com("https://api.github.com/"));
        assert!(!is_github_dot_com("https://github.com.evil.example"));
    }

    /// Red if this returns `true` unconditionally: a text-only terminal must
    /// never trigger a fetch whose result it cannot show.
    #[test]
    fn images_are_not_fetched_on_a_terminal_that_cannot_draw_them() {
        // Covers `Protocol::None`, which is what every case in
        // `terminal_image`'s own capability tests reduces to: a pane
        // multiplexer (herdr/tmux/zellij), a plain terminal, and
        // `ui.graph_renderer = "text"` alike.
        assert!(!should_fetch_images(
            AvatarPlacement::Author,
            capability(Protocol::None)
        ));
    }

    #[test]
    fn a_badge_never_fails_even_for_an_empty_identity() {
        let b = badge("", "");
        assert_eq!(b.initials, "?");
        assert_eq!(badge_glyph(&b), "?");
    }

    #[test]
    fn the_badge_glyph_is_the_initials_first_character_not_byte() {
        let b = badge("Śebastien Géiser", "s@example.com");
        assert_eq!(b.initials, "ŚG");
        assert_eq!(badge_glyph(&b), "Ś");
    }

    #[test]
    fn the_same_author_always_gets_the_same_badge() {
        assert_eq!(
            badge("Ada Lovelace", "ada@example.com"),
            badge("Ada Lovelace", "ada@example.com")
        );
    }
}
