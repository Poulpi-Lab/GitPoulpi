//! History rewrite scripts (the history-rewrite design,
//! "Scripts"): the per-commit Lua script a `Script` rule runs.
//!
//! - `contract.rs`: the JSON request / response a script speaks;
//! - `lua_rt.rs`: the sandboxed Lua 5.4 runtime (limits, determinism);
//! - `external.rs`: shell, Python and external Lua scripts as a process
//!   speaking the same contract as JSON Lines (S11);
//! - `stage.rs`: loading, checking, running over a pass, testing.
//!
//! Behind the `rewrite-scripts` feature: without it every entry point is a
//! typed `Error::Unsupported`.

#[cfg(feature = "rewrite-scripts")]
mod contract;
#[cfg(feature = "rewrite-scripts")]
mod external;
#[cfg(feature = "rewrite-scripts")]
mod lua_rt;
#[cfg(feature = "rewrite-scripts")]
mod stage;

#[cfg(feature = "rewrite-scripts")]
pub(crate) use stage::{check, load, run_stage, supported, test, ScriptEnv};

#[cfg(not(feature = "rewrite-scripts"))]
mod off {
    use std::path::Path;

    use gitpoulpi_core::domain::rewrite::script::{ScriptCheck, ScriptTestRow};
    use gitpoulpi_core::domain::rewrite::{ScriptPolicy, ScriptRef};
    use gitpoulpi_core::domain::Oid;
    use gitpoulpi_core::{Error, Result};

    use crate::history::rules::Work;

    const OFF: &str = "rewrite scripts (built without the `rewrite-scripts` feature)";

    /// Never built: every constructor refuses.
    #[allow(dead_code)]
    pub(crate) struct Script;

    fn off<T>() -> Result<T> {
        Err(Error::Unsupported { operation: OFF })
    }

    /// What the real module asks for; unused here.
    #[derive(Clone, Copy)]
    #[allow(dead_code)]
    pub(crate) struct ScriptEnv<'a> {
        pub policy: &'a ScriptPolicy,
        pub probe: &'a crate::tools::ToolProbe,
    }

    pub(crate) fn supported(_: &ScriptRef) -> Result<()> {
        off()
    }
    pub(crate) fn check(_: &Path, _: &ScriptRef, _: ScriptEnv<'_>) -> Result<ScriptCheck> {
        off()
    }
    pub(crate) fn load(_: &Path, _: &ScriptRef, _: ScriptEnv<'_>) -> Result<Script> {
        off()
    }
    pub(crate) fn test(
        _: &Path,
        _: &ScriptRef,
        _: ScriptEnv<'_>,
        _: &[Oid],
    ) -> Result<Vec<ScriptTestRow>> {
        off()
    }
    pub(crate) fn run_stage(
        _: &Script,
        _: &mut [Work],
        _: &[Oid],
        _: &[bool],
        _: &mut dyn FnMut(u64) -> Result<()>,
        _: &mut dyn FnMut(usize, Vec<String>),
    ) -> Result<u64> {
        off()
    }
}

#[cfg(not(feature = "rewrite-scripts"))]
pub(crate) use off::{check, load, run_stage, supported, test, ScriptEnv};
