//! `gitpoulpi mcp token`: the bearer token of the MCP HTTP endpoint.
//!
//! Not an MCP tool (`gitpoulpi-mcp`'s `EXCLUDED`): an agent must not read or
//! replace the key that guards its own server.

use std::io::{IsTerminal, Write};

use gitpoulpi_config::{mcp_secrets, ConfigPaths};
use serde_json::{json, Value};

/// Creates the token on first use, rotates it when asked, and reports
/// `{token_hint, path, rotated}`. `show` adds the full `token`; `copy` puts
/// it on the clipboard through the terminal (OSC 52) and adds
/// `copied: true`, without printing it.
pub fn token(paths: &ConfigPaths, show: bool, copy: bool, rotate: bool) -> Result<Value, String> {
    let io = |e: std::io::Error| format!("cannot keep the MCP token: {e}");
    let token = if rotate {
        mcp_secrets::rotate_token(paths).map_err(io)?
    } else {
        mcp_secrets::ensure_token(paths).map_err(io)?.0
    };
    let mut report = json!({
        "token_hint": mcp_secrets::token_hint(&token),
        "path": mcp_secrets::token_file(paths).display().to_string(),
        "rotated": rotate,
    });
    if show {
        report["token"] = Value::String(token.clone());
    }
    if copy {
        copy_to_terminal_clipboard(&token)?;
        report["copied"] = Value::Bool(true);
    }
    Ok(report)
}

fn copy_to_terminal_clipboard(text: &str) -> Result<(), String> {
    let mut err = std::io::stderr();
    if !err.is_terminal() {
        return Err("--copy needs a terminal to reach the clipboard through; use --show".into());
    }
    err.write_all(mcp_secrets::osc52(text).as_bytes())
        .and_then(|()| err.flush())
        .map_err(|e| format!("cannot reach the terminal: {e}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_report_hides_the_token_unless_asked() {
        let dir = tempfile::tempdir().unwrap();
        let paths = ConfigPaths::for_dir(dir.path());
        let plain = token(&paths, false, false, false).unwrap();
        assert!(plain.get("token").is_none(), "{plain}");
        assert_eq!(plain["rotated"], false);
        let shown = token(&paths, true, false, false).unwrap();
        let full = shown["token"].as_str().unwrap().to_string();
        assert_eq!(shown["token_hint"], mcp_secrets::token_hint(&full));
        let rotated = token(&paths, true, false, true).unwrap();
        assert_eq!(rotated["rotated"], true);
        assert_ne!(rotated["token"].as_str().unwrap(), full);
    }
}
