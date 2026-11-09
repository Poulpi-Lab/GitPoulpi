//! `xtask emit <hex> [--hold <ms>]`: write exactly these bytes to stdout,
//! then stay alive.
//!
//! A deliberately boring child for the pty driver's own self-tests, which
//! need a program whose output is predictable to the byte. The obvious
//! candidates are not portable: `/bin/sh -c "printf ..."` forces
//! `#[cfg(unix)]` (no driver coverage on Windows), and `cmd /c echo`
//! appends its own line ending and mangles carets and quotes. So the child
//! is this workspace's own binary, spawned by `CARGO_BIN_EXE_xtask`.
//!
//! Hex rather than a literal string because the interesting payloads are
//! escape sequences, which quoting rules of every shell would mangle.

use std::io::Write;
use std::time::Duration;

pub const USAGE: &str = "\
Usage: cargo run -p xtask -- emit <hex> [--hold <ms>] [--exit <code>]

Writes the hex-decoded bytes to stdout, then sleeps for --hold (default
30000) so the pty stays open for a test to read, then exits with --exit
(default 0). An empty payload writes nothing, which is how the driver's
tests exercise a child that is alive but silent. Used by the pty
driver's own self-tests; not interesting on its own.";

pub fn run(args: &[String]) -> Result<i32, String> {
    let mut payload: Option<&str> = None;
    let mut hold = Duration::from_millis(30_000);
    let mut exit_code = 0i32;
    let mut rest = args.iter();

    while let Some(arg) = rest.next() {
        match arg.as_str() {
            "-h" | "--help" => return Err(USAGE.to_string()),
            "--hold" => {
                let value = rest.next().ok_or("--hold needs a number of ms")?;
                hold = Duration::from_millis(
                    value.parse().map_err(|_| "--hold needs a number of ms")?,
                );
            }
            "--exit" => {
                let value = rest.next().ok_or("--exit needs a code")?;
                exit_code = value.parse().map_err(|_| "--exit needs a code")?;
            }
            other if payload.is_none() => payload = Some(other),
            other => return Err(format!("unexpected argument {other:?}\n\n{USAGE}")),
        }
    }

    let payload = payload.ok_or_else(|| format!("no payload given\n\n{USAGE}"))?;
    let bytes = decode_hex(payload)?;

    let mut stdout = std::io::stdout();
    stdout.write_all(&bytes).map_err(|e| e.to_string())?;
    stdout.flush().map_err(|e| e.to_string())?;

    // Held open rather than exiting: the driver's reader sees EOF the
    // moment the child's end of the pty closes, and a test that wants to
    // assert on a *live* screen needs the child still attached. The test
    // kills it; the hold is only a backstop against a leaked process.
    std::thread::sleep(hold);
    Ok(exit_code)
}

fn decode_hex(text: &str) -> Result<Vec<u8>, String> {
    let digits: Vec<char> = text.chars().filter(|c| !c.is_whitespace()).collect();
    if !digits.len().is_multiple_of(2) {
        return Err("hex payload must have an even number of digits".to_string());
    }
    digits
        .chunks(2)
        .map(|pair| {
            let byte: String = pair.iter().collect();
            u8::from_str_radix(&byte, 16).map_err(|_| format!("{byte:?} is not a hex byte"))
        })
        .collect()
}

/// Encodes `bytes` for this task's argument. Public so the driver's
/// tests can build a payload without reimplementing it - and so the
/// round trip is covered by the test below rather than assumed.
pub fn encode_hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hex_round_trips_including_control_bytes() {
        let payload = b"\x1b[1;31mRED\x1b[0m\r\n";
        assert_eq!(decode_hex(&encode_hex(payload)).unwrap(), payload);
    }

    #[test]
    fn a_malformed_payload_is_an_error_rather_than_a_truncated_write() {
        assert!(decode_hex("abc").is_err());
        assert!(decode_hex("zz").is_err());
    }
}
