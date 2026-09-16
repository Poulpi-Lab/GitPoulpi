use std::path::PathBuf;

use crate::{FixtureRepo, FixtureRepoBuilder};

/// `FixtureRepoBuilder` for test code: every step panics (with the
/// caller's location) instead of returning a `Result`, so a fixture reads
/// as one chain with no `.unwrap()` between steps.
pub struct TestRepoBuilder(FixtureRepoBuilder);

macro_rules! steps {
    ($($(#[$doc:meta])* $name:ident($($arg:ident: $ty:ty),*);)+) => {
        $(
            $(#[$doc])*
            #[track_caller]
            pub fn $name(self $(, $arg: $ty)*) -> Self {
                Self(self.0.$name($($arg),*).expect(concat!("fixture step `", stringify!($name), "` failed")))
            }
        )+
    };
}

impl TestRepoBuilder {
    #[track_caller]
    pub fn new() -> Self {
        Self(FixtureRepoBuilder::new().expect("fixture repo creation failed"))
    }

    /// A reftable repository, or `None` (with a printed note) when the
    /// installed git cannot create one: the caller returns early.
    pub fn reftable() -> Option<Self> {
        match FixtureRepoBuilder::new_reftable() {
            Ok(b) => Some(Self(b)),
            Err(e) => {
                eprintln!("SKIPPED: this git cannot create a reftable repository ({e})");
                None
            }
        }
    }

    steps! {
        /// `commit("initial")`, the start of most fixtures.
        with_initial();
        commit(message: &str);
        commit_with_body(summary: &str, body: &str);
        commit_bytes(path: &str, contents: &[u8], message: &str);
        commit_png(path: &str, w: u32, h: u32, rgba: &[u8], message: &str);
        commit_file(path: &str, contents: &str, message: &str);
        commit_pointer_text(path: &str, oid: &str, size: u64, message: &str);
        commit_removal(path: &str, message: &str);
        write_worktree(path: &str, contents: &[u8]);
        stage(path: &str);
        config(key: &str, value: &str);
        branch(name: &str);
        checkout(name: &str);
        annotated_tag(name: &str);
        with_origin();
        with_remote(name: &str);
        with_submodule(path: &str);
        with_lfs();
        lfs_track(pattern: &str);
        lfs_track_lockable(pattern: &str);
        commit_lfs_file(path: &str, contents: &[u8], message: &str);
        with_lfs_origin();
        push_to(name: &str, refspec: &str);
        track(branch: &str, remote: &str);
        tag(name: &str);
        push(refspec: &str);
    }

    #[track_caller]
    pub fn worktree(self, name: &str, branch: &str) -> (Self, PathBuf) {
        let (builder, path) = self
            .0
            .worktree(name, branch)
            .expect("fixture step `worktree` failed");
        (Self(builder), path)
    }

    pub fn build(self) -> FixtureRepo {
        self.0.build()
    }
}

impl Default for TestRepoBuilder {
    fn default() -> Self {
        Self::new()
    }
}
