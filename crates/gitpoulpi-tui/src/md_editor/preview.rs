//! The Preview of the Markdown editor: the text laid out by `md_render`,
//! kept for the width and theme it was made for.
//!
//! Short texts are laid out on the spot (the budget is 4 ms for 10 KB).
//! Above [`SYNC_LIMIT`] the layout goes to a worker thread that waits for
//! the typing to pause ([`DEBOUNCE`]) and answers through the redraw poke,
//! so the render thread never waits on a long text; until the answer
//! arrives the previous layout stays on screen with an "updating" mark.

use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::sync::mpsc::{self, Receiver, Sender};
use std::sync::Arc;
use std::time::Duration;

use crate::md_render::{self, Context, Doc};
use crate::redraw::NotifySend as _;

/// Texts up to this many bytes are laid out on the render thread.
pub(crate) const SYNC_LIMIT: usize = 16 * 1024;
/// How long a worker waits for the typing to pause before it lays out.
pub(crate) const DEBOUNCE: Duration = Duration::from_millis(120);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Key {
    stamp: u64,
    width: usize,
    theme: u64,
}

struct Job {
    key: Key,
    text: String,
    ctx: Context,
}

struct Done {
    key: Key,
    doc: Doc,
}

struct Worker {
    tx: Sender<Job>,
    rx: Receiver<Done>,
}

impl Worker {
    fn spawn() -> Self {
        let (tx, jobs) = mpsc::channel::<Job>();
        let (done_tx, rx) = mpsc::channel::<Done>();
        std::thread::spawn(move || {
            while let Ok(mut job) = jobs.recv() {
                // Typing goes on: only the newest text is worth laying out.
                loop {
                    match jobs.recv_timeout(DEBOUNCE) {
                        Ok(newer) => job = newer,
                        Err(_) => break,
                    }
                }
                let doc = md_render::render(&job.text, job.key.width, &job.ctx);
                if done_tx.send_notify(Done { key: job.key, doc }).is_err() {
                    return;
                }
            }
        });
        Self { tx, rx }
    }
}

fn stamp(text: &str) -> u64 {
    let mut h = DefaultHasher::new();
    text.hash(&mut h);
    h.finish()
}

/// The laid-out text of one editor.
#[derive(Default)]
pub(crate) struct Preview {
    shown: Option<(Key, Arc<Doc>)>,
    /// The layout asked of the worker and not back yet.
    wanted: Option<Key>,
    worker: Option<Worker>,
    /// First visible line.
    pub(crate) top: usize,
    pub(crate) bar: Option<crate::scrollbar::Track>,
    pub(crate) drag: crate::scrollbar::BarDrag,
}

impl Preview {
    /// Takes in what the worker has finished.
    pub(crate) fn poll(&mut self) {
        let Some(worker) = &self.worker else {
            return;
        };
        while let Ok(done) = worker.rx.try_recv() {
            if self.wanted == Some(done.key) {
                self.wanted = None;
            }
            self.shown = Some((done.key, Arc::new(done.doc)));
        }
    }

    /// The layout of `text` at `width` cells and whether it is the layout
    /// of an older text (a worker is still busy). `None` before the first
    /// layout of a long text arrives.
    pub(crate) fn doc(
        &mut self,
        text: &str,
        width: usize,
        ctx: &Context,
    ) -> (Option<Arc<Doc>>, bool) {
        self.poll();
        let key = Key {
            stamp: stamp(text),
            width,
            theme: crate::theme::theme_generation(),
        };
        if let Some((k, doc)) = &self.shown {
            if *k == key {
                return (Some(Arc::clone(doc)), false);
            }
        }
        if text.len() <= SYNC_LIMIT {
            let doc = Arc::new(md_render::render(text, width, ctx));
            self.shown = Some((key, Arc::clone(&doc)));
            self.wanted = None;
            return (Some(doc), false);
        }
        if self.wanted != Some(key) {
            let worker = self.worker.get_or_insert_with(Worker::spawn);
            let _ = worker.tx.send(Job {
                key,
                text: text.to_string(),
                ctx: ctx.clone(),
            });
            self.wanted = Some(key);
        }
        (self.shown.as_ref().map(|(_, d)| Arc::clone(d)), true)
    }

    /// Whether a layout is on its way.
    pub(crate) fn updating(&self) -> bool {
        self.wanted.is_some()
    }
}
