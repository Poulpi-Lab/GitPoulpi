//! Port checks for the MCP HTTP endpoint (`mcp.bind`): is the address taken,
//! and which port next to it is free. Several installations default to the
//! same bind, so Settings offers a free one and `mcp serve` refuses a taken
//! one. Both do real bind attempts; they can block on a slow host name
//! lookup, so the TUI runs them off the render thread.

use std::io::ErrorKind;
use std::net::{SocketAddr, TcpListener, ToSocketAddrs};

use crate::McpConfig;

/// How many ports above the current one [`suggest_free_bind`] tries before it
/// lets the OS pick.
const SCAN: u32 = 200;

fn split(bind: &str) -> (String, Option<u16>) {
    let cfg = McpConfig {
        bind: bind.to_string(),
        ..McpConfig::default()
    };
    cfg.bind_parts()
}

fn resolve(host: &str, port: u16) -> Option<SocketAddr> {
    let host = host.trim_start_matches('[').trim_end_matches(']');
    (host, port).to_socket_addrs().ok()?.next()
}

/// Whether something already listens on `bind` (a real bind attempt that
/// fails with "address in use"). An unreadable address is not "in use".
pub fn bind_in_use(bind: &str) -> bool {
    let (host, port) = split(bind);
    let Some(addr) = port.and_then(|p| resolve(&host, p)) else {
        return false;
    };
    TcpListener::bind(addr).is_err_and(|e| e.kind() == ErrorKind::AddrInUse)
}

/// `bind` with a free port, the host kept: the first bindable port above the
/// current one, else any port the OS gives. `None` when nothing could be
/// bound on that host.
pub fn suggest_free_bind(bind: &str) -> Option<String> {
    let (host, port) = split(bind);
    let start = u32::from(port.unwrap_or(crate::MCP_DEFAULT_PORT));
    for candidate in (start + 1..=start + SCAN).filter_map(|p| u16::try_from(p).ok()) {
        if resolve(&host, candidate).is_some_and(|a| TcpListener::bind(a).is_ok()) {
            return Some(format!("{host}:{candidate}"));
        }
    }
    let listener = TcpListener::bind(resolve(&host, 0)?).ok()?;
    Some(format!("{host}:{}", listener.local_addr().ok()?.port()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_suggestion_keeps_the_host_and_is_bindable() {
        let held = TcpListener::bind("127.0.0.1:0").unwrap();
        let taken = held.local_addr().unwrap().port();
        let bind = format!("127.0.0.1:{taken}");
        assert!(bind_in_use(&bind));
        let suggested = suggest_free_bind(&bind).unwrap();
        let (host, port) = suggested.rsplit_once(':').unwrap();
        assert_eq!(host, "127.0.0.1");
        let port: u16 = port.parse().unwrap();
        assert_ne!(port, taken);
        assert!(TcpListener::bind(("127.0.0.1", port)).is_ok());
    }

    #[test]
    fn a_free_bind_is_not_in_use_and_garbage_is_not_either() {
        let free = {
            let l = TcpListener::bind("127.0.0.1:0").unwrap();
            l.local_addr().unwrap().port()
        };
        assert!(!bind_in_use(&format!("127.0.0.1:{free}")));
        assert!(!bind_in_use("nonsense"));
    }
}
