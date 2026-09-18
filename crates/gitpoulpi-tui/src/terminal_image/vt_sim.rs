//! A tiny terminal model for tests: a grid of cells, each with the text
//! last written there and the iTerm2 picture tile on it. It follows the
//! rule real terminals (WezTerm, iTerm2) follow and that the image layer
//! relies on: writing a character into a cell drops the picture tile that
//! cell carried, and an inline picture puts a tile on every cell of its
//! `width` x `height` box starting at the cursor.
//!
//! It reads exactly the bytes the TUI writes: ratatui's diff of two
//! buffers ([`Vt::draw`]) and the escape sequences of the image layer
//! ([`Vt::feed`]: `ESC 7`/`ESC 8`, `CSI r;c H`, SGR ignored, `OSC 1337`).

use ratatui::buffer::Buffer;
use ratatui::layout::Rect;

pub(crate) struct Vt {
    width: u16,
    height: u16,
    text: Vec<String>,
    tile: Vec<Option<u32>>,
    cursor: (u16, u16),
    saved: (u16, u16),
    pictures: u32,
}

impl Vt {
    pub(crate) fn new(area: Rect) -> Vt {
        let n = usize::from(area.width) * usize::from(area.height);
        Vt {
            width: area.width,
            height: area.height,
            text: vec![" ".to_string(); n],
            tile: vec![None; n],
            cursor: (0, 0),
            saved: (0, 0),
            pictures: 0,
        }
    }

    fn at(&self, x: u16, y: u16) -> Option<usize> {
        (x < self.width && y < self.height)
            .then(|| usize::from(y) * usize::from(self.width) + usize::from(x))
    }

    fn put(&mut self, x: u16, y: u16, symbol: &str) {
        if let Some(i) = self.at(x, y) {
            self.text[i] = symbol.to_string();
            self.tile[i] = None;
        }
    }

    /// ratatui's flush: only the cells that differ, `skip` ones never.
    pub(crate) fn draw(&mut self, prev: &Buffer, next: &Buffer) {
        for (x, y, cell) in prev.diff(next) {
            self.put(x, y, cell.symbol());
        }
    }

    /// The picture tile on a cell (the picture's sequence number).
    pub(crate) fn tile(&self, x: u16, y: u16) -> Option<u32> {
        self.at(x, y).and_then(|i| self.tile[i])
    }

    /// Every cell of `area` that carries a picture tile.
    pub(crate) fn tiles_in(&self, area: Rect) -> Vec<(u16, u16)> {
        let mut out = Vec::new();
        for y in area.top()..area.bottom() {
            for x in area.left()..area.right() {
                if self.tile(x, y).is_some() {
                    out.push((x, y));
                }
            }
        }
        out
    }

    pub(crate) fn feed(&mut self, bytes: &[u8]) {
        let s = String::from_utf8_lossy(bytes).into_owned();
        let mut it = s.chars().peekable();
        while let Some(c) = it.next() {
            match c {
                '\x1b' => match it.next() {
                    Some('7') => self.saved = self.cursor,
                    Some('8') => self.cursor = self.saved,
                    Some('[') => {
                        let mut params = String::new();
                        let mut last = ' ';
                        for p in it.by_ref() {
                            if p.is_ascii_alphabetic() {
                                last = p;
                                break;
                            }
                            params.push(p);
                        }
                        if last == 'H' {
                            let mut n = params.split(';').map(|v| v.parse::<u16>().unwrap_or(1));
                            let (row, col) = (n.next().unwrap_or(1), n.next().unwrap_or(1));
                            self.cursor = (col - 1, row - 1);
                        }
                    }
                    Some('P') => {
                        // A DCS string (sixel) runs to `ESC \`.
                        let mut body = String::new();
                        while let Some(p) = it.next() {
                            if p == '\x1b' {
                                it.next();
                                break;
                            }
                            body.push(p);
                        }
                        self.sixel(&body);
                    }
                    Some(']') => {
                        let mut body = String::new();
                        for p in it.by_ref() {
                            if p == '\x07' {
                                break;
                            }
                            body.push(p);
                        }
                        self.picture(&body);
                    }
                    _ => {}
                },
                c if c.is_control() => {}
                c => {
                    let (x, y) = self.cursor;
                    self.put(x, y, &c.to_string());
                    self.cursor.0 += 1;
                }
            }
        }
    }

    /// A sixel puts a tile on every cell its pixels touch, from the
    /// cursor: the raster attributes (`"1;1;W;H`) give the size, the cell
    /// size is the one the encoder used.
    fn sixel(&mut self, body: &str) {
        let Some(raster) = body.split('"').nth(1) else {
            return;
        };
        let mut n = raster
            .split(|c: char| !c.is_ascii_digit() && c != ';')
            .next()
            .unwrap_or("")
            .split(';')
            .skip(2)
            .map(|v| v.parse::<u32>().unwrap_or(0));
        let (w, h) = (n.next().unwrap_or(0), n.next().unwrap_or(0));
        let cell = super::cell_size();
        let (cols, rows) = (w.div_ceil(cell.width), h.div_ceil(cell.height));
        self.pictures += 1;
        let id = self.pictures;
        let (x0, y0) = self.cursor;
        for y in y0..y0 + rows as u16 {
            for x in x0..x0 + cols as u16 {
                if let Some(i) = self.at(x, y) {
                    self.tile[i] = Some(id);
                }
            }
        }
    }

    fn picture(&mut self, body: &str) {
        if !body.starts_with("1337;File=") {
            return;
        }
        let args = body.split(':').next().unwrap_or("");
        let get = |key: &str| {
            args.split(';')
                .find_map(|kv| kv.strip_prefix(key))
                .and_then(|v| v.parse::<u16>().ok())
                .unwrap_or(1)
        };
        let (w, h) = (get("width="), get("height="));
        self.pictures += 1;
        let id = self.pictures;
        let (x0, y0) = self.cursor;
        for y in y0..y0 + h {
            for x in x0..x0 + w {
                if let Some(i) = self.at(x, y) {
                    self.tile[i] = Some(id);
                }
            }
        }
    }
}
