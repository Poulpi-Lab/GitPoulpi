//! The MCP HTTP endpoint's security rows on Settings > Agents & MCP: HTTP
//! authentication, the access token (masked; the value shows or hides it,
//! with Copy and Rotate buttons), TLS, the certificate and key paths, the certificate
//! fingerprint, and the "behind a tunnel or reverse proxy" statement.
//!
//! The switches and paths are ordinary `[mcp]` keys; the token is not a
//! config value but a user-only file (`gitpoulpi_config::mcp_secrets`) that
//! the server re-reads on every request, so a rotation applies at once. The
//! token never reaches the status line or a log: Copy goes through
//! `clipboard::copy_secret_shared`, which echoes nothing.
//!
//! Keyboard twins: the buttons are one `Buttons` row (Left/Right pick, Enter
//! presses); the other rows are the generic toggle, choice and text rows.
//! Every row is found by the Settings search (`/`, or *Search Settings* in
//! the palette).

use super::*;
use gitpoulpi_config::{mcp_secrets, McpTls};
use gitpoulpi_i18n::msg::mcp_security as t;

/// A button of the access token row.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum McpButton {
    /// Reveal or hide the token on screen (the masked value is the button).
    Show,
    /// Copy the token to the clipboard.
    Copy,
    /// Replace the token (asks first: clients holding the old one stop working).
    Rotate,
    /// Make the first token.
    Create,
}

/// What the rows remember between frames.
pub(super) struct McpSecurityState {
    /// Where the secrets live (`None` when the OS gave no config folder).
    pub(super) paths: Option<ConfigPaths>,
    /// The token as last read or written.
    pub(super) token: Option<String>,
    /// The token is drawn in full.
    pub(super) shown: bool,
}

impl McpSecurityState {
    pub(super) fn read() -> Self {
        let paths = ConfigPaths::discover();
        let token = paths.as_ref().and_then(mcp_secrets::read_token);
        Self {
            paths,
            token,
            shown: false,
        }
    }
}

impl PreferencesState {
    /// The `[mcp]` switches, choice and paths, unplaced (the row pool).
    pub(super) fn mcp_security_rows(&self) -> Vec<Field> {
        let mcp = &self.config.mcp;
        let mut auth = Field::global(
            t::AUTH.get(),
            "mcp.http_auth",
            Control::Toggle(mcp.http_auth),
        )
        .with_help(t::AUTH_HELP.get());
        if !mcp.http_auth {
            auth = auth.with_danger(t::AUTH_WARNING.get());
        }
        let tls = Field::global(
            t::TLS.get(),
            "mcp.tls",
            Control::Choice {
                options: McpTls::ALL.iter().map(|m| m.as_str().into()).collect(),
                index: McpTls::ALL.iter().position(|m| *m == mcp.tls).unwrap_or(0),
            },
        )
        .with_help(t::TLS_HELP.get());
        let cert = Field::global(
            t::TLS_CERT.get(),
            "mcp.tls_cert",
            Control::Text(mcp.tls_cert.clone()),
        )
        .with_help(t::TLS_CERT_HELP.get());
        let key = Field::global(
            t::TLS_KEY.get(),
            "mcp.tls_key",
            Control::Text(mcp.tls_key.clone()),
        )
        .with_help(t::TLS_KEY_HELP.get());
        let mut proxy = Field::global(
            t::PROXY.get(),
            "mcp.behind_proxy",
            Control::Toggle(mcp.behind_proxy),
        )
        .with_help(t::PROXY_HELP.get());
        if mcp.behind_proxy {
            proxy = proxy.with_danger(t::PROXY_WARNING.get());
        }
        let mut rows = vec![auth, self.mcp_token_row()];
        // Shown in full on a row of its own: a long value cannot fit the
        // button beside Copy and Rotate. `picked` keeps an info row with no
        // target next to the row before it.
        if let (true, Some(token)) = (self.mcp_sec.shown, &self.mcp_sec.token) {
            rows.push(Field::plain(
                &format!("  {}", t::TOKEN.get()),
                Control::Info(token.clone()),
                Target::None,
            ));
        }
        rows.extend([tls, cert, key, proxy]);
        rows
    }

    /// The access token row: the token masked (itself a button that shows
    /// it in full on the row below, or hides it again) and Copy and Rotate. Its key
    /// `mcp.token` is virtual (like `mcp.port`): it is no config value, it
    /// gives the palette and the Settings search a name for the row.
    fn mcp_token_row(&self) -> Field {
        let state = &self.mcp_sec;
        let has = state.token.is_some();
        let shown = match &state.token {
            None => t::TOKEN_NONE.get().to_string(),
            Some(token) => mcp_secrets::token_hint(token),
        };
        let mut value = Btn::new(&shown, PrefAction::McpSecurity(McpButton::Show));
        value.fill = true;
        value.enabled = has;
        let mut copy = Btn::new(t::COPY.get(), PrefAction::McpSecurity(McpButton::Copy));
        copy.enabled = has;
        let rotate = if has {
            Btn::new(t::ROTATE.get(), PrefAction::McpSecurity(McpButton::Rotate))
        } else {
            Btn::new(t::CREATE.get(), PrefAction::McpSecurity(McpButton::Create))
        };
        let mut field = Field::buttons(t::TOKEN.get(), vec![value, copy, rotate])
            .with_help(t::TOKEN_HELP.get());
        field.target = Target::Global("mcp.token");
        field
    }

    /// The certificate fingerprint, right under the key path.
    pub(super) fn mcp_fingerprint_rows(&self) -> Vec<Field> {
        let mcp = &self.config.mcp;
        let text = match mcp.tls {
            McpTls::Off => t::FINGERPRINT_OFF.get().to_string(),
            McpTls::SelfSigned => match self.mcp_sec.paths.as_ref() {
                Some(paths) => {
                    let cert = mcp_secrets::self_signed_cert(paths);
                    if !cert.exists() {
                        t::FINGERPRINT_PENDING.get().to_string()
                    } else {
                        fingerprint_text(&cert)
                    }
                }
                None => t::FINGERPRINT_PENDING.get().to_string(),
            },
            McpTls::Files => fingerprint_text(std::path::Path::new(mcp.tls_cert.trim())),
        };
        vec![
            Field::plain(t::FINGERPRINT.get(), Control::Info(text), Target::None)
                .with_help(t::FINGERPRINT_HELP.get()),
        ]
    }

    pub(super) fn mcp_button(&mut self, button: McpButton) {
        match button {
            McpButton::Show => {
                if self.mcp_sec.token.is_some() {
                    self.mcp_sec.shown = !self.mcp_sec.shown;
                }
            }
            McpButton::Copy => {
                let Some(token) = self.mcp_sec.token.clone() else {
                    self.message = Some(t::TOKEN_MISSING.get().into());
                    return;
                };
                self.message = Some(match crate::clipboard::copy_secret_shared(&token) {
                    Ok(()) => t::TOKEN_COPIED.get().into(),
                    Err(why) => why,
                });
            }
            McpButton::Rotate | McpButton::Create => {
                let Some(paths) = self.mcp_sec.paths.as_ref() else {
                    self.message = Some(
                        gitpoulpi_i18n::msg::settings_screen::NO_CONFIG_DIR
                            .get()
                            .into(),
                    );
                    return;
                };
                match mcp_secrets::rotate_token(paths) {
                    Ok(token) => {
                        self.mcp_sec.token = Some(token);
                        self.message = Some(t::TOKEN_ROTATED.get().into());
                    }
                    Err(err) => self.message = Some(t::token_failed(err)),
                }
            }
        }
    }
}

fn fingerprint_text(cert: &std::path::Path) -> String {
    mcp_secrets::fingerprint_of_file(cert).unwrap_or_else(|| t::FINGERPRINT_UNREADABLE.get().into())
}
