//! A crossterm backend that writes nothing when nothing changed.
//!
//! `Terminal::draw` hands the backend an empty diff on a frame that looks
//! like the last one, and the stock backend still answers with a style reset
//! and a cursor hide (24 bytes). An idle TUI must write 0 bytes, so this
//! wrapper drops the empty draw and the cursor commands that restate the
//! known state. Anything written through [`Write`] (kitty pictures, raw
//! escapes) may move the cursor or change its visibility, so it forgets what
//! it knew.
//!
//! A synchronized-update bracket ([`QuietBackend::begin_group`]) is held back
//! until the frame writes something, and dropped with its end if it never does.

use std::io::{self, Write};

use ratatui::backend::{Backend, ClearType, CrosstermBackend, WindowSize};
use ratatui::buffer::Cell;
use ratatui::layout::{Position, Size};

pub struct QuietBackend<W: Write> {
    inner: CrosstermBackend<W>,
    /// `Some(true)`: known hidden. Unknown after any raw write.
    cursor_hidden: Option<bool>,
    /// The cursor position we last set and nothing has moved since.
    cursor_at: Option<Position>,
    /// Bytes to emit before the next write of any kind.
    prefix: Vec<u8>,
    /// Whether the frame wrote anything since `begin_group`.
    group_active: bool,
    group_wrote: bool,
}

impl<W: Write> QuietBackend<W> {
    pub fn new(writer: W) -> Self {
        Self {
            inner: CrosstermBackend::new(writer),
            cursor_hidden: None,
            cursor_at: None,
            prefix: Vec::new(),
            group_active: false,
            group_wrote: false,
        }
    }

    /// Starts a bracket: `prefix` goes out right before the first real write.
    pub fn begin_group(&mut self, prefix: &[u8]) {
        self.prefix = prefix.to_vec();
        self.group_active = true;
        self.group_wrote = false;
    }

    /// Ends the bracket: `suffix` is written only if the prefix was.
    pub fn end_group(&mut self, suffix: &[u8]) -> io::Result<()> {
        let wrote = self.group_wrote;
        self.group_active = false;
        self.group_wrote = false;
        self.prefix.clear();
        if wrote {
            self.inner.write_all(suffix)?;
            Write::flush(&mut self.inner)?;
            self.forget();
        }
        Ok(())
    }

    fn forget(&mut self) {
        self.cursor_hidden = None;
        self.cursor_at = None;
    }

    fn emit_prefix(&mut self) -> io::Result<()> {
        if self.group_active && !self.group_wrote {
            self.group_wrote = true;
            let prefix = std::mem::take(&mut self.prefix);
            self.inner.write_all(&prefix)?;
        }
        Ok(())
    }
}

impl<W: Write> Write for QuietBackend<W> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        if buf.is_empty() {
            return Ok(0);
        }
        self.emit_prefix()?;
        self.forget();
        self.inner.write(buf)
    }

    fn flush(&mut self) -> io::Result<()> {
        Write::flush(&mut self.inner)
    }
}

impl<W: Write> Backend for QuietBackend<W> {
    fn draw<'a, I>(&mut self, content: I) -> io::Result<()>
    where
        I: Iterator<Item = (u16, u16, &'a Cell)>,
    {
        let mut content = content.peekable();
        if content.peek().is_none() {
            return Ok(());
        }
        self.emit_prefix()?;
        self.cursor_at = None;
        self.inner.draw(content)
    }

    fn hide_cursor(&mut self) -> io::Result<()> {
        if self.cursor_hidden == Some(true) {
            return Ok(());
        }
        self.emit_prefix()?;
        self.cursor_hidden = Some(true);
        self.inner.hide_cursor()
    }

    fn show_cursor(&mut self) -> io::Result<()> {
        if self.cursor_hidden == Some(false) {
            return Ok(());
        }
        self.emit_prefix()?;
        self.cursor_hidden = Some(false);
        self.inner.show_cursor()
    }

    fn get_cursor_position(&mut self) -> io::Result<Position> {
        self.inner.get_cursor_position()
    }

    fn set_cursor_position<P: Into<Position>>(&mut self, position: P) -> io::Result<()> {
        let position = position.into();
        if self.cursor_at == Some(position) {
            return Ok(());
        }
        self.emit_prefix()?;
        self.cursor_at = Some(position);
        self.inner.set_cursor_position(position)
    }

    fn clear(&mut self) -> io::Result<()> {
        self.emit_prefix()?;
        self.forget();
        self.inner.clear()
    }

    fn clear_region(&mut self, clear_type: ClearType) -> io::Result<()> {
        self.emit_prefix()?;
        self.forget();
        self.inner.clear_region(clear_type)
    }

    fn append_lines(&mut self, n: u16) -> io::Result<()> {
        self.emit_prefix()?;
        self.forget();
        self.inner.append_lines(n)
    }

    fn size(&self) -> io::Result<Size> {
        self.inner.size()
    }

    fn window_size(&mut self) -> io::Result<WindowSize> {
        self.inner.window_size()
    }

    fn flush(&mut self) -> io::Result<()> {
        Write::flush(&mut self.inner)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::Terminal;
    use std::sync::{Arc, Mutex};

    #[derive(Clone, Default)]
    struct Sink(Arc<Mutex<Vec<u8>>>);

    impl Sink {
        fn len(&self) -> usize {
            self.0.lock().unwrap().len()
        }
    }

    impl Write for Sink {
        fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
            self.0.lock().unwrap().extend_from_slice(buf);
            Ok(buf.len())
        }
        fn flush(&mut self) -> io::Result<()> {
            Ok(())
        }
    }

    #[test]
    fn an_unchanged_frame_writes_nothing() {
        let sink = Sink::default();
        let mut t = Terminal::with_options(
            QuietBackend::new(sink.clone()),
            ratatui::TerminalOptions {
                viewport: ratatui::Viewport::Fixed(ratatui::layout::Rect::new(0, 0, 10, 2)),
            },
        )
        .unwrap();
        let draw = |t: &mut Terminal<QuietBackend<Sink>>| {
            t.draw(|f| f.render_widget(ratatui::widgets::Paragraph::new("hi"), f.area()))
                .unwrap();
        };
        draw(&mut t);
        let first = sink.len();
        assert!(first > 0);
        draw(&mut t);
        draw(&mut t);
        assert_eq!(sink.len(), first);
    }

    #[test]
    fn a_group_with_no_writes_is_dropped() {
        let sink = Sink::default();
        let mut b = QuietBackend::new(sink.clone());
        b.begin_group(b"<");
        b.end_group(b">").unwrap();
        assert_eq!(sink.len(), 0);
        b.begin_group(b"<");
        b.write_all(b"x").unwrap();
        b.end_group(b">").unwrap();
        assert_eq!(&*sink.0.lock().unwrap(), b"<x>");
    }
}
