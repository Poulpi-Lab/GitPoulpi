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
    /// Look for a free port on the bind's host and save it as this
    /// installation's MCP bind.
    SuggestPort,
}

/// What the rows remember between frames.
pub(super) struct McpSecurityState {
    /// Where the secrets live (`None` when the OS gave no config folder).
    pub(super) paths: Option<ConfigPaths>,
    /// The token as last read or written.
    pub(super) token: Option<String>,
    /// The token is drawn in full.
    pub(super) shown: bool,
    /// The free-port search running off the render thread.
    pub(super) suggest: Option<std::sync::mpsc::Receiver<Option<String>>>,
}

impl McpSecurityState {
    pub(super) fn read() -> Self {
        let paths = ConfigPaths::discover();
        let token = paths.as_ref().and_then(mcp_secrets::read_token);
        Self {
            paths,
            token,
            shown: false,
            suggest: None,
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
                    Ok(crate::clipboard_route::Delivery::Terminal) => {
                        gitpoulpi_i18n::msg::input::copied_via_terminal(t::TOKEN.get())
                    }
                    Ok(_) => t::TOKEN_COPIED.get().into(),
                    Err(why) => why,
                });
            }
            McpButton::SuggestPort => self.suggest_port_start(),
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

impl PreferencesState {
    /// The bind this installation serves on: its own override when it has
    /// one, else `[mcp] bind`.
    pub(super) fn mcp_bind_in_force(&self) -> String {
        let own = self.own_entry().mcp_bind;
        if own.trim().is_empty() {
            self.config.mcp.bind.clone()
        } else {
            own
        }
    }

    /// `[mcp]` as this installation sees it: its bind in force.
    pub(super) fn mcp_in_force(&self) -> gitpoulpi_config::McpConfig {
        let mut mcp = self.config.mcp.clone();
        mcp.bind = self.mcp_bind_in_force();
        mcp
    }

    /// The Suggest a free port row, right under the port row.
    pub(super) fn mcp_suggest_row(&self) -> Field {
        let mut btn = Btn::new(
            t::SUGGEST_PORT.get(),
            PrefAction::McpSecurity(McpButton::SuggestPort),
        );
        btn.enabled = self.mcp_sec.suggest.is_none();
        let mut field = Field::buttons(t::SUGGEST_PORT.get(), vec![btn]).with_help(t::SUGGEST_PORT_HELP.get());
        field.target = Target::Global("mcp.suggest_port");
        field
    }

    /// The palette's route: show the row, then run the search.
    pub(crate) fn mcp_suggest_command(&mut self) {
        let _ = self.show_page(None, Some("mcp.suggest_port"));
        self.mcp_button(McpButton::SuggestPort);
    }

    /// Looks for the port on a thread (a host name lookup can block).
    fn suggest_port_start(&mut self) {
        if self.mcp_sec.suggest.is_some() {
            return;
        }
        let bind = self.mcp_bind_in_force();
        let (tx, rx) = std::sync::mpsc::channel();
        self.mcp_sec.suggest = Some(rx);
        self.message = Some(t::SUGGEST_PORT_WORKING.get().into());
        std::thread::spawn(move || {
            let _ = tx.send_notify(gitpoulpi_config::mcp_port::suggest_free_bind(&bind));
        });
    }

    /// Applies a finished search. Never blocks; called once a frame.
    pub(crate) fn poll_mcp_suggest(&mut self) {
        let Some(rx) = &self.mcp_sec.suggest else {
            return;
        };
        let Ok(found) = rx.try_recv() else { return };
        self.mcp_sec.suggest = None;
        match found {
            Some(bind) => {
                if self.write_mcp_bind(&bind) {
                    self.message = Some(t::suggested(&bind));
                }
            }
            None => self.message = Some(t::SUGGEST_PORT_FAILED.get().into()),
        }
    }

    /// Saves `bind` where this installation reads it: its own entry (the
    /// primary only when it already has an override), else `mcp.bind`.
    /// Byte-preserving, like the other installation rows.
    pub(super) fn write_mcp_bind(&mut self, bind: &str) -> bool {
        let Some(file) = self.config_file() else {
            self.message = Some(
                gitpoulpi_i18n::msg::settings_screen::NO_CONFIG_DIR
                    .get()
                    .into(),
            );
            return false;
        };
        let seeded = self.own_entry();
        let own = !seeded.mcp_bind.trim().is_empty();
        if self.this_effective().primary && !own {
            return self.write_global("mcp.bind", bind);
        }
        let identity = self.this().identity.clone();
        let id = identity.id.clone();
        let edit = |i: &mut gitpoulpi_config::InstallationConfig| {
            let entry = i.settings.entry(id.clone()).or_insert_with(|| {
                if seeded.path.is_empty() {
                    gitpoulpi_config::InstallationEntry {
                        path: identity.root.clone(),
                        kind: identity.kind,
                        ..Default::default()
                    }
                } else {
                    seeded.clone()
                }
            });
            entry.mcp_bind = bind.to_string();
        };
        if let Err(e) = gitpoulpi_config::set_installation_in_file(&file, edit) {
            self.message = Some(format!("installation.mcp_bind: {e}"));
            return false;
        }
        self.config = Config::load(&file).unwrap_or_else(|_| self.config.clone());
        true
    }
}

fn fingerprint_text(cert: &std::path::Path) -> String {
    mcp_secrets::fingerprint_of_file(cert).unwrap_or_else(|| t::FINGERPRINT_UNREADABLE.get().into())
}
