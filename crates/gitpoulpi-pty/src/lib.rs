//! A child process on an emulated terminal.
//!
//! [`PtySession`] spawns a program on a pty (a ConPTY on Windows, behind
//! the same `portable-pty` API), feeds everything it paints to a `vt100`
//! screen, answers the questions a terminal is expected to answer (see
//! [`answer_queries`]) and lets the caller write keystrokes and read the
//! screen back. It is the implementation behind the AI terminal mode
//! (`gitpoulpi-ai`); the test driver (`xtask::pty`) shares
//! [`answer_queries`] and [`RepExpander`], the parts that took a Windows CI
//! runner to get right.
//!
//! No `cfg(windows)` in the API: the one place the platforms differ is
//! teardown, documented at [`PtySession::stop`].

mod query;
mod rep;
mod session;

pub use query::{answer_queries, QUERY_CARRY};
pub use rep::RepExpander;
pub use session::{PtySession, SpawnOptions};
