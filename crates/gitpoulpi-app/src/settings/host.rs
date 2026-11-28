//! The rows that belong to the machine and the running installation: the
//! installation's name, colour and role, the launcher, the command aliases and
//! the MCP server's security.
//!
//! What they hold depends on state only the running frontend reads (which
//! installation this is, the shortcuts found on the machine, the stored access
//! token), so the frontend that edits them builds the rows with their values.
//! This module is their **descriptor**: the same keys, labels, kinds and
//! order, with the values of a machine with nothing special - enough to list
//! them (the palette, the search, a second frontend's own editors) and to
//! check that the frontend's rows have not drifted from it.

use super::{Action, Btn, Control, PoolCtx, Row, Target};
use gitpoulpi_i18n::msg::{launcher as t, mcp_security as mt};

fn host_button(label: &str, key: &'static str) -> Row {
    let mut row = Row::plain(
        label,
        Control::Buttons(vec![Btn::new(label, Action::Host(key))]),
        Target::Global(key),
    );
    row.live = true;
    row
}

/// The "This installation" rows: info, name, version in the name, colour,
/// primary, environment and folder-menu submenu.
pub fn installation_rows(_cx: &PoolCtx) -> Vec<Row> {
    let combo = |custom: bool| Control::Combo {
        options: Vec::new(),
        value: String::new(),
        custom,
    };
    vec![
        Row::global(
            t::INSTALLATION.get(),
            "installation.info",
            Control::Info(String::new()),
        )
        .live(),
        Row::global(
            t::NAME.get(),
            "installation.name",
            Control::Text(String::new()),
        )
        .live(),
        Row::global(
            t::NAME_WITH_VERSION.get(),
            "installation.name_with_version",
            Control::Toggle(false),
        )
        .live(),
        Row::global(t::COLOR.get(), "installation.logo_color", combo(false)).live(),
        Row::global(
            t::PRIMARY.get(),
            "installation.primary",
            Control::Toggle(false),
        )
        .live(),
        Row::global(t::ENV.get(), "installation.env", combo(true)).live(),
        Row::global(
            t::SUBMENU.get(),
            "installation.folder_menu_submenu",
            combo(true),
        )
        .live(),
    ]
}

/// The launcher rows: terminal, relaunch, reuse window, the two command
/// aliases and (developer mode only) the build choice.
pub fn launcher_rows(cx: &PoolCtx) -> Vec<Row> {
    let mut relaunch = Row::global(
        t::RELAUNCH.get(),
        "ui.relaunch_in_terminal",
        Control::Toggle(false),
    );
    if !cfg!(windows) {
        relaunch = relaunch.coming(t::WINDOWS_ONLY.get());
    }
    let mut rows = vec![
        Row::global(
            t::TERMINAL.get(),
            "ui.launch_terminal",
            Control::Combo {
                options: Vec::new(),
                value: String::new(),
                custom: false,
            },
        ),
        relaunch,
        Row::global(t::REUSE.get(), "ui.reuse_window", Control::Toggle(false)),
        Row::global(
            t::ALIAS.get(),
            "installation.command_alias",
            Control::Combo {
                options: Vec::new(),
                value: String::new(),
                custom: true,
            },
        )
        .live(),
        Row::global(
            t::ALIAS_ADD.get(),
            "installation.command_alias_add",
            Control::Combo {
                options: Vec::new(),
                value: String::new(),
                custom: true,
            },
        )
        .live(),
    ];
    if gitpoulpi_config::dev_mode::dev_mode(cx.config, &std::env::current_exe().unwrap_or_default())
    {
        rows.push(Row::global(
            t::BUILD.get(),
            "ui.launcher_build",
            Control::Combo {
                options: Vec::new(),
                value: String::new(),
                custom: false,
            },
        ));
    }
    rows
}

/// The "Suggest a free port" button, right under the port row.
pub fn mcp_suggest_row(_cx: &PoolCtx) -> Row {
    host_button(mt::SUGGEST_PORT.get(), "mcp.suggest_port")
}

/// The `[mcp]` switches, token and TLS rows, unplaced.
pub fn mcp_security_rows(_cx: &PoolCtx) -> Vec<Row> {
    use gitpoulpi_config::McpTls;
    vec![
        Row::global(mt::AUTH.get(), "mcp.http_auth", Control::Toggle(false)),
        host_button(mt::TOKEN.get(), "mcp.token"),
        Row::global(
            mt::TLS.get(),
            "mcp.tls",
            Control::Choice {
                options: McpTls::ALL.iter().map(|m| m.as_str().into()).collect(),
                index: 0,
            },
        ),
        Row::global(
            mt::TLS_CERT.get(),
            "mcp.tls_cert",
            Control::Text(String::new()),
        ),
        Row::global(
            mt::TLS_KEY.get(),
            "mcp.tls_key",
            Control::Text(String::new()),
        ),
        Row::global(mt::PROXY.get(), "mcp.behind_proxy", Control::Toggle(false)),
    ]
}
