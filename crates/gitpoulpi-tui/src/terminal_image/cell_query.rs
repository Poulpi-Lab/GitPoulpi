//! The cell's pixel size on Windows, asked of the terminal (`CSI 16 t`),
//! for **sixel**: a sixel has no "fit to N cells", its pixels are the size,
//! so a wrong guess puts the picture out of its cells (and over a tall
//! graph column, out of its rows). Windows has no `TIOCGWINSZ` and
//! `FALLBACK` (9x21) is only right for one font.
//!
//! The reason [`super::cell_size`] never reads stdin is a helper thread
//! blocked in `read` that cannot be stopped. This does not use one: it
//! waits on the console input handle with a deadline (`WaitForSingleObject`)
//! and reads the records itself, on the thread that owns input, so nothing
//! outlives the call. Windows Terminal answers in about 10 ms. A terminal
//! that does not answer costs one deadline, then is never asked again.

use std::sync::atomic::{AtomicU32, Ordering};

use super::CellSize;

/// Packed `width << 16 | height`; 0 = not measured.
static MEASURED: AtomicU32 = AtomicU32::new(0);

/// The size the last successful query reported.
pub(super) fn measured() -> Option<CellSize> {
    let packed = MEASURED.load(Ordering::Relaxed);
    (packed != 0).then(|| CellSize::sanitised(packed >> 16, packed & 0xffff))
}

#[cfg_attr(not(any(windows, test)), allow(dead_code))]
fn store(width: u32, height: u32) {
    MEASURED.store(
        (width.min(0xffff) << 16) | height.min(0xffff),
        Ordering::Relaxed,
    );
}

/// `ESC [ 6 ; height ; width t` -> `(width, height)` in pixels.
#[cfg_attr(not(any(windows, test)), allow(dead_code))]
pub(super) fn parse_report(reply: &str) -> Option<(u32, u32)> {
    let body = reply.split("\x1b[6;").nth(1)?.split('t').next()?;
    let (h, w) = body.split_once(';')?;
    let (w, h) = (w.trim().parse::<u32>().ok()?, h.trim().parse::<u32>().ok()?);
    (w > 0 && h > 0).then_some((w, h))
}

/// Asks the terminal for its cell size once more (a zoom changes it).
/// `true` when a size is now known. A no-op, `false`, off Windows.
pub(crate) fn measure() -> bool {
    #[cfg(windows)]
    {
        windows::measure()
    }
    #[cfg(not(windows))]
    {
        false
    }
}

#[cfg(windows)]
mod windows {
    use super::*;
    use std::io::Write;
    use std::sync::atomic::AtomicBool;
    use std::time::{Duration, Instant};

    const DEADLINE: Duration = Duration::from_millis(200);
    const STD_INPUT_HANDLE: u32 = -10i32 as u32;
    const KEY_EVENT: u16 = 1;

    /// A terminal that never answered: not asked again.
    static SILENT: AtomicBool = AtomicBool::new(false);

    #[repr(C)]
    #[derive(Clone, Copy)]
    struct KeyEventRecord {
        key_down: i32,
        repeat: u16,
        virtual_key: u16,
        scan: u16,
        ch: u16,
        control: u32,
    }

    #[repr(C)]
    #[derive(Clone, Copy)]
    union Event {
        key: KeyEventRecord,
        raw: [u32; 4],
    }

    #[repr(C)]
    #[derive(Clone, Copy)]
    struct InputRecord {
        kind: u16,
        event: Event,
    }

    extern "system" {
        fn GetStdHandle(which: u32) -> isize;
        fn WaitForSingleObject(handle: isize, ms: u32) -> u32;
        fn ReadConsoleInputW(
            handle: isize,
            records: *mut InputRecord,
            length: u32,
            read: *mut u32,
        ) -> i32;
    }

    pub(super) fn measure() -> bool {
        if SILENT.load(Ordering::Relaxed) {
            return super::measured().is_some();
        }
        // SAFETY: a plain handle lookup.
        let input = unsafe { GetStdHandle(STD_INPUT_HANDLE) };
        if input == 0 || input == -1 {
            return false;
        }
        let mut out = std::io::stdout();
        if out
            .write_all(b"\x1b[16t")
            .and_then(|()| out.flush())
            .is_err()
        {
            return false;
        }
        let started = Instant::now();
        let mut reply = String::new();
        while started.elapsed() < DEADLINE && !reply.ends_with('t') {
            let left = DEADLINE.saturating_sub(started.elapsed()).as_millis() as u32;
            // SAFETY: waits on a handle the process owns.
            if unsafe { WaitForSingleObject(input, left) } != 0 {
                break;
            }
            let mut record = InputRecord {
                kind: 0,
                event: Event { raw: [0; 4] },
            };
            let mut read = 0u32;
            // SAFETY: one record's room, `read` receives the count.
            let ok = unsafe { ReadConsoleInputW(input, &mut record, 1, &mut read) };
            if ok == 0 || read == 0 {
                break;
            }
            if record.kind == KEY_EVENT {
                // SAFETY: `kind` says the key variant is the live one.
                let key = unsafe { record.event.key };
                if key.key_down != 0 && key.ch != 0 {
                    if let Some(c) = char::from_u32(u32::from(key.ch)) {
                        reply.push(c);
                    }
                }
            }
        }
        match parse_report(&reply) {
            Some((w, h)) => {
                store(w, h);
                true
            }
            None => {
                SILENT.store(true, Ordering::Relaxed);
                super::measured().is_some()
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_report_is_height_then_width() {
        assert_eq!(parse_report("\x1b[6;20;10t"), Some((10, 20)));
        // Typed characters before the reply do not matter.
        assert_eq!(parse_report("ab\x1b[6;21;9t"), Some((9, 21)));
    }

    #[test]
    fn nonsense_and_zero_are_refused() {
        assert_eq!(parse_report(""), None);
        assert_eq!(parse_report("\x1b[6;0;0t"), None);
        assert_eq!(parse_report("\x1b[4;600;1200t"), None);
        assert_eq!(parse_report("\x1b[6;20t"), None);
    }

    #[test]
    fn a_stored_size_is_what_cell_size_reports() {
        store(10, 20);
        assert_eq!(measured(), Some(CellSize::sanitised(10, 20)));
        MEASURED.store(0, Ordering::Relaxed);
    }
}
