//! The embedded Lua 5.4 runtime of history rewrite scripts.
//!
//! What a script can do is what this file lets it:
//!
//! - **No outside world.** Only the `string`, `table`, `math` and `utf8`
//!   libraries are opened: no `io`, `os`, `package` (so no `require`),
//!   `debug` or `coroutine`. The base functions that read files, compile
//!   text or bytecode (`dofile`, `loadfile`, `load`), print or catch errors
//!   (`pcall`, `xpcall`: a script could otherwise swallow the limit error
//!   and loop on) are removed, and so are `string.dump`, `collectgarbage`
//!   and the clock-seeded `math.random` / `math.randomseed`.
//! - **Deterministic.** Lua orders the keys of a table by a hash seeded from
//!   the clock and from addresses, so `pairs` is replaced by one that
//!   visits keys in sorted order, `next` answers only "the first key" (to
//!   test for an empty table) and `tostring` never prints an address.
//!   Response tables are read back with sorted keys. The only randomness is
//!   `rand(seed)`, a pure function of its argument.
//! - **Bounded.** An instruction-count hook (every 1000 instructions) stops a
//!   call after [`Limits::instructions`] instructions or [`Limits::call_time`]
//!   of wall time, and the allocator refuses to grow past [`Limits::memory`].
//!   Time is the safety net, the instruction count the reproducible limit.
//!   A single long call into a C library function (a pathological
//!   `string.find` pattern) is not interrupted by the hook.

use std::cell::{Cell, RefCell};
use std::rc::Rc;
use std::time::{Duration, Instant};

use gitpoulpi_core::domain::rewrite::trailers::{self, TrailerIfExists};
use mlua::{
    ChunkMode, DeserializeOptions, Function, HookTriggers, Lua, LuaOptions, LuaSerdeExt,
    SerializeOptions, StdLib, Table, Value as LuaValue, VmState,
};
use regex::RegexBuilder;
use serde_json::Value;

/// The functions a script may define.
pub(super) const ENTRY_POINTS: [&str; 3] = ["rewrite_commit", "rewrite_blob", "rewrite_tag"];

const HOOK_EVERY: u32 = 1000;
const MAX_LOG_LINES: usize = 64;
const MAX_LOG_BYTES: usize = 1024;
const REGEX_SIZE_LIMIT: usize = 1 << 20;

#[derive(Debug, Clone, Copy)]
pub(super) struct Limits {
    /// Lua VM instructions per call.
    pub(super) instructions: u64,
    /// Bytes the Lua state may grow by after the script is loaded.
    pub(super) memory: usize,
    /// Wall time per call.
    pub(super) call_time: Duration,
}

impl Default for Limits {
    fn default() -> Self {
        Self {
            instructions: 10_000_000,
            memory: 64 << 20,
            call_time: Duration::from_secs(5),
        }
    }
}

/// Runs before the unsafe base functions are removed.
const PRELUDE: &str = r#"
local rawnext, type, sort, ts, error = next, type, table.sort, tostring, error
local rank = { boolean = 1, number = 2, string = 3 }
local function lt(a, b)
  local ta, tb = type(a), type(b)
  if ta ~= tb then return rank[ta] < rank[tb] end
  if ta == "boolean" then return (not a) and b end
  return a < b
end
local function ordered(t)
  if type(t) ~= "table" then
    error("bad argument #1 to 'pairs' (table expected, got " .. type(t) .. ")", 3)
  end
  local keys, n = {}, 0
  for k in rawnext, t do
    if not rank[type(k)] then
      error("pairs: a table key must be a boolean, a number or a string", 3)
    end
    n = n + 1
    keys[n] = k
  end
  sort(keys, lt)
  local i = 0
  return function()
    i = i + 1
    if i <= n then
      local k = keys[i]
      return k, t[k]
    end
  end
end
function pairs(t) return ordered(t), t, nil end
function next(t, k)
  if k ~= nil then error("next: only next(t) is available, use pairs to walk a table", 2) end
  return ordered(t)()
end
function tostring(v)
  local ty = type(v)
  if ty == "table" or ty == "function" or ty == "thread" or ty == "userdata" then return ty end
  return ts(v)
end
"#;

pub(super) struct Runtime {
    lua: Lua,
    limits: Limits,
    ops: Rc<Cell<u64>>,
    started: Rc<Cell<Instant>>,
    logs: Rc<RefCell<Vec<String>>>,
}

/// The first line(s) of a Lua error, without the traceback.
fn clean(error: &mlua::Error) -> String {
    let mut e = error;
    while let mlua::Error::CallbackError { cause, .. } = e {
        e = cause;
    }
    let text = e.to_string();
    let text = text.split("\nstack traceback:").next().unwrap_or(&text);
    text.trim().to_string()
}

fn lua_err(e: mlua::Error) -> String {
    clean(&e)
}

impl Runtime {
    /// Opens the sandbox and runs the script's top level (which defines its
    /// functions). The same limits apply to that run.
    pub(super) fn new(source: &str, limits: Limits) -> Result<Self, String> {
        let libs = StdLib::STRING | StdLib::TABLE | StdLib::MATH | StdLib::UTF8;
        let lua = Lua::new_with(libs, LuaOptions::new()).map_err(lua_err)?;
        let rt = Self {
            lua,
            limits,
            ops: Rc::default(),
            started: Rc::new(Cell::new(Instant::now())),
            logs: Rc::default(),
        };
        rt.sandbox()?;
        rt.helpers()?;
        // Hook and memory limit after the setup: the limits are for scripts.
        rt.install_limits()?;
        rt.arm();
        rt.lua
            .load(source)
            .set_name("=script")
            .set_mode(ChunkMode::Text)
            .exec()
            .map_err(lua_err)?;
        Ok(rt)
    }

    fn sandbox(&self) -> Result<(), String> {
        self.lua
            .load(PRELUDE)
            .set_name("=prelude")
            .exec()
            .map_err(lua_err)?;
        let g = self.lua.globals();
        for name in [
            "dofile",
            "loadfile",
            "load",
            "loadstring",
            "print",
            "warn",
            "collectgarbage",
            "pcall",
            "xpcall",
            "require",
            "package",
            "io",
            "os",
            "debug",
            "coroutine",
        ] {
            g.set(name, LuaValue::Nil).map_err(lua_err)?;
        }
        let string: Table = g.get("string").map_err(lua_err)?;
        string.set("dump", LuaValue::Nil).map_err(lua_err)?;
        let math: Table = g.get("math").map_err(lua_err)?;
        math.set("random", LuaValue::Nil).map_err(lua_err)?;
        math.set("randomseed", LuaValue::Nil).map_err(lua_err)?;
        Ok(())
    }

    /// `regex_replace`, `regex_match`, `stable_hash`, `rand`, `weekday`,
    /// `hour` and `log`: pure functions of their arguments (and the log).
    fn helpers(&self) -> Result<(), String> {
        let lua = &self.lua;
        let g = lua.globals();
        let set = |name: &str, f: Function| g.set(name, f).map_err(lua_err);

        set(
            "regex_replace",
            lua.create_function(|_, (text, pattern, with): (String, String, String)| {
                let re = compile(&pattern)?;
                Ok(re.replace_all(&text, with.as_str()).into_owned())
            })
            .map_err(lua_err)?,
        )?;
        set(
            "regex_match",
            lua.create_function(|_, (text, pattern): (String, String)| {
                Ok(compile(&pattern)?.is_match(&text))
            })
            .map_err(lua_err)?,
        )?;
        set(
            "stable_hash",
            lua.create_function(|_, (text, salt): (String, Option<String>)| {
                Ok(stable_hash(&text, salt.as_deref().unwrap_or("")))
            })
            .map_err(lua_err)?,
        )?;
        set(
            "rand",
            lua.create_function(|_, seed: i64| Ok(rand(seed)))
                .map_err(lua_err)?,
        )?;
        set(
            "weekday",
            lua.create_function(|_, (time, offset): (i64, Option<i64>)| {
                Ok(weekday(time, offset.unwrap_or(0)))
            })
            .map_err(lua_err)?,
        )?;
        set(
            "hour",
            lua.create_function(|_, (time, offset): (i64, Option<i64>)| {
                Ok(hour(time, offset.unwrap_or(0)))
            })
            .map_err(lua_err)?,
        )?;
        // Trailers of a message (the grammar of `interpret-trailers`), and
        // edits that keep every other byte: `set_trailer` replaces what the
        // key had (a fourth argument `"add"` or `"skip"` changes that).
        set(
            "trailers",
            lua.create_function(|lua, message: String| {
                let out = lua.create_table()?;
                for t in trailers::parse(&message) {
                    let row = lua.create_table()?;
                    row.set("key", t.key)?;
                    row.set("value", t.value)?;
                    out.push(row)?;
                }
                Ok(out)
            })
            .map_err(lua_err)?,
        )?;
        set(
            "set_trailer",
            lua.create_function(
                |_, (message, key, value, how): (String, String, String, Option<String>)| {
                    let how = match how.as_deref() {
                        None | Some("replace") => TrailerIfExists::Replace,
                        Some("add") => TrailerIfExists::Add,
                        Some("skip") => TrailerIfExists::Skip,
                        Some(other) => {
                            return Err(mlua::Error::runtime(format!(
                                "set_trailer: `{other}` is not replace, add or skip"
                            )))
                        }
                    };
                    if key.is_empty()
                        || !key.chars().all(|c| c.is_ascii_alphanumeric() || c == '-')
                        || value.contains(['\n', '\r'])
                    {
                        return Err(mlua::Error::runtime(
                            "set_trailer: a key is letters, digits and `-`, a value one line",
                        ));
                    }
                    Ok(trailers::add(&message, &key, &value, how))
                },
            )
            .map_err(lua_err)?,
        )?;
        set(
            "remove_trailer",
            lua.create_function(|_, (message, key): (String, String)| {
                Ok(trailers::remove(&message, &key, None))
            })
            .map_err(lua_err)?,
        )?;
        set(
            "at",
            lua.create_function(|_, (text, offset): (String, Option<i64>)| {
                at(&text, offset.unwrap_or(0)).map_err(mlua::Error::runtime)
            })
            .map_err(lua_err)?,
        )?;
        let logs = Rc::clone(&self.logs);
        set(
            "log",
            lua.create_function(move |_, text: String| {
                let mut logs = logs.borrow_mut();
                if logs.len() < MAX_LOG_LINES {
                    let mut line = text;
                    if line.len() > MAX_LOG_BYTES {
                        let mut cut = MAX_LOG_BYTES;
                        while !line.is_char_boundary(cut) {
                            cut -= 1;
                        }
                        line.truncate(cut);
                    }
                    logs.push(line);
                }
                Ok(())
            })
            .map_err(lua_err)?,
        )?;
        Ok(())
    }

    fn install_limits(&self) -> Result<(), String> {
        let (ops, started) = (Rc::clone(&self.ops), Rc::clone(&self.started));
        let (max, time) = (self.limits.instructions, self.limits.call_time);
        self.lua
            .set_global_hook(
                HookTriggers::new().every_nth_instruction(HOOK_EVERY),
                move |_, _| {
                    let n = ops.get() + u64::from(HOOK_EVERY);
                    ops.set(n);
                    if n > max {
                        return Err(mlua::Error::runtime(format!(
                            "too long: the script ran more than {max} instructions in one call"
                        )));
                    }
                    if started.get().elapsed() > time {
                        return Err(mlua::Error::runtime(format!(
                            "too long: the script ran for more than {} ms in one call",
                            time.as_millis()
                        )));
                    }
                    Ok(VmState::Continue)
                },
            )
            .map_err(lua_err)?;
        self.lua
            .set_memory_limit(self.lua.used_memory() + self.limits.memory)
            .map_err(lua_err)?;
        Ok(())
    }

    /// A new call starts with a full budget and an empty log.
    fn arm(&self) {
        self.ops.set(0);
        self.started.set(Instant::now());
        self.logs.borrow_mut().clear();
    }

    /// The entry points the script defines.
    pub(super) fn functions(&self) -> Vec<String> {
        let g = self.lua.globals();
        ENTRY_POINTS
            .iter()
            .filter(|n| matches!(g.get::<LuaValue>(**n), Ok(LuaValue::Function(_))))
            .map(|n| n.to_string())
            .collect()
    }

    /// Calls `function` with `args` (JSON values become Lua tables). The
    /// answer is the returned value as JSON (`Null` for nothing) and the
    /// lines the call logged. Errors are the Lua error text.
    pub(super) fn call(
        &self,
        function: &str,
        args: &[Value],
    ) -> Result<(Value, Vec<String>), String> {
        let f: Function = self
            .lua
            .globals()
            .get(function)
            .map_err(|_| format!("the script defines no `{function}`"))?;
        let options = SerializeOptions::new()
            .set_array_metatable(false)
            .serialize_none_to_null(false)
            .serialize_unit_to_null(false);
        let lua_args = args
            .iter()
            .map(|a| self.lua.to_value_with(a, options))
            .collect::<Result<Vec<_>, _>>()
            .map_err(lua_err)?;
        self.arm();
        let out: LuaValue = f
            .call(mlua::MultiValue::from_iter(lua_args))
            .map_err(lua_err)?;
        let value = match out {
            LuaValue::Nil => Value::Null,
            other => self
                .lua
                .from_value_with(other, DeserializeOptions::new().sort_keys(true))
                .map_err(|e| format!("the returned value cannot be used: {}", clean(&e)))?,
        };
        let logs = self.logs.borrow().clone();
        Ok((value, logs))
    }
}

fn compile(pattern: &str) -> mlua::Result<regex::Regex> {
    RegexBuilder::new(pattern)
        .size_limit(REGEX_SIZE_LIMIT)
        .build()
        .map_err(|e| mlua::Error::runtime(format!("bad regex: {e}")))
}

/// FNV-1a over `salt`, a separator and `text`, cut to 53 bits so the result
/// is the same integer in every Lua number type.
fn stable_hash(text: &str, salt: &str) -> i64 {
    let mut h: u64 = 0xcbf2_9ce4_8422_2325;
    for b in salt.bytes().chain(std::iter::once(0)).chain(text.bytes()) {
        h ^= u64::from(b);
        h = h.wrapping_mul(0x0000_0100_0000_01b3);
    }
    (h & ((1 << 53) - 1)) as i64
}

/// SplitMix64 of the seed as a float in `[0, 1)`.
fn rand(seed: i64) -> f64 {
    let mut z = (seed as u64).wrapping_add(0x9e37_79b9_7f4a_7c15);
    z = (z ^ (z >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
    z ^= z >> 31;
    (z >> 11) as f64 / (1u64 << 53) as f64
}

/// ISO weekday (1 = Monday .. 7 = Sunday) of a Unix time in a UTC offset
/// given in minutes.
fn weekday(time: i64, offset_minutes: i64) -> i64 {
    let days = (time + offset_minutes * 60).div_euclid(86_400);
    // 1970-01-01 was a Thursday.
    (days + 3).rem_euclid(7) + 1
}

/// Unix time of `YYYY-MM-DD[ T]HH:MM[:SS]` (the time may be left out) read
/// in a UTC offset given in minutes east.
fn at(text: &str, offset_minutes: i64) -> Result<i64, String> {
    let bad = || format!("at: `{text}` is not YYYY-MM-DD[ HH:MM[:SS]]");
    let text = text.trim();
    let (date, time) = match text.split_once([' ', 'T']) {
        Some((d, t)) => (d, t.trim()),
        None => (text, ""),
    };
    let number = |s: &str| s.parse::<i64>().map_err(|_| bad());
    let d: Vec<&str> = date.split('-').collect();
    let [y, m, day] = d[..] else {
        return Err(bad());
    };
    let (y, m, day) = (number(y)?, number(m)?, number(day)?);
    let t: Vec<&str> = if time.is_empty() {
        vec![]
    } else {
        time.split(':').collect()
    };
    let (hh, mm, ss) = match t[..] {
        [] => (0, 0, 0),
        [h, m] => (number(h)?, number(m)?, 0),
        [h, m, s] => (number(h)?, number(m)?, number(s)?),
        _ => return Err(bad()),
    };
    let days_in_month = match m {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 if (y % 4 == 0 && y % 100 != 0) || y % 400 == 0 => 29,
        2 => 28,
        _ => return Err(bad()),
    };
    if !(1..=days_in_month).contains(&day)
        || !(0..24).contains(&hh)
        || !(0..60).contains(&mm)
        || !(0..60).contains(&ss)
        || !(1..=9999).contains(&y)
        || offset_minutes.abs() > 24 * 60
    {
        return Err(bad());
    }
    // Days since 1970-01-01 (Hinnant's `days_from_civil`).
    let y2 = if m <= 2 { y - 1 } else { y };
    let era = y2.div_euclid(400);
    let yoe = y2 - era * 400;
    let doy = (153 * (if m > 2 { m - 3 } else { m + 9 }) + 2) / 5 + day - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    let days = era * 146_097 + doe - 719_468;
    Ok(days * 86_400 + hh * 3600 + mm * 60 + ss - offset_minutes * 60)
}

fn hour(time: i64, offset_minutes: i64) -> i64 {
    (time + offset_minutes * 60).rem_euclid(86_400) / 3600
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn run(source: &str, arg: Value) -> Result<Value, String> {
        let rt = Runtime::new(source, Limits::default())?;
        rt.call("rewrite_commit", &[arg, json!({})]).map(|r| r.0)
    }

    fn small() -> Limits {
        Limits {
            instructions: 200_000,
            memory: 8 << 20,
            call_time: Duration::from_secs(5),
        }
    }

    fn fails(source: &str) -> String {
        let rt = Runtime::new(source, small()).expect("the script loads");
        rt.call("rewrite_commit", &[json!({}), json!({})])
            .expect_err("the call must fail")
    }

    #[test]
    fn a_table_goes_in_and_a_table_comes_out() {
        let out = run(
            "function rewrite_commit(c) return { message = c.message .. '!', n = 2 } end",
            json!({"message": "hi"}),
        )
        .unwrap();
        assert_eq!(out, json!({"message": "hi!", "n": 2}));
    }

    #[test]
    fn nothing_returned_is_null() {
        assert_eq!(
            run("function rewrite_commit(c) end", json!({})).unwrap(),
            Value::Null
        );
    }

    #[test]
    fn io_os_and_require_are_gone() {
        for code in [
            "io.open('/etc/passwd')",
            "os.time()",
            "os.getenv('HOME')",
            "require('os')",
            "dofile('x.lua')",
            "loadfile('x.lua')",
            "load('return 1')",
            "package.path = ''",
            "debug.getinfo(1)",
            "string.dump(print)",
            "collectgarbage('count')",
            "math.random()",
            "math.randomseed(1)",
            "pcall(print)",
            "coroutine.create(print)",
        ] {
            let e = fails(&format!("function rewrite_commit() {code} end"));
            assert!(e.contains("nil") || e.contains("attempt to"), "{code}: {e}");
        }
    }

    #[test]
    fn an_instruction_loop_is_stopped() {
        let e = fails("function rewrite_commit() while true do end end");
        assert!(e.contains("too long") && e.contains("instructions"), "{e}");
    }

    #[test]
    fn the_limit_cannot_be_swallowed() {
        // There is no pcall to catch the limit error with.
        let e = fails(
            "function rewrite_commit() while true do local ok = pcall(function() while true do end end) end end",
        );
        assert!(e.contains("nil") || e.contains("pcall"), "{e}");
    }

    #[test]
    fn the_budget_is_per_call() {
        let rt = Runtime::new(
            "function rewrite_commit() local n = 0 for i = 1, 50000 do n = n + 1 end return { n = n } end",
            small(),
        )
        .unwrap();
        for _ in 0..20 {
            assert_eq!(
                rt.call("rewrite_commit", &[json!({}), json!({})])
                    .unwrap()
                    .0,
                json!({"n": 50000})
            );
        }
    }

    #[test]
    fn runaway_recursion_is_an_error_not_a_crash() {
        let rt = Runtime::new(
            "function f(n) return 1 + f(n + 1) end function rewrite_commit() return f(1) end",
            Limits::default(),
        )
        .unwrap();
        let e = rt
            .call("rewrite_commit", &[json!({}), json!({})])
            .unwrap_err();
        assert!(e.contains("stack overflow"), "{e}");
    }

    #[test]
    fn memory_is_capped() {
        let e = fails("function rewrite_commit() local s = string.rep('x', 1 << 30) return s end");
        assert!(e.to_lowercase().contains("memory"), "{e}");
        let e =
            fails("function rewrite_commit() local t = {} for i = 1, 1e7 do t[i] = {i} end end");
        assert!(
            e.to_lowercase().contains("memory") || e.contains("too long"),
            "{e}"
        );
    }

    #[test]
    fn wall_time_is_capped() {
        let rt = Runtime::new(
            "function rewrite_commit() while true do end end",
            Limits {
                instructions: u64::MAX / 2,
                memory: 8 << 20,
                call_time: Duration::from_millis(50),
            },
        )
        .unwrap();
        let e = rt
            .call("rewrite_commit", &[json!({}), json!({})])
            .unwrap_err();
        assert!(e.contains("ms in one call"), "{e}");
    }

    #[test]
    fn syntax_errors_name_the_line() {
        let e = Runtime::new("function rewrite_commit(\n", small())
            .err()
            .unwrap();
        assert!(e.contains("script:"), "{e}");
    }

    #[test]
    fn bytecode_is_refused() {
        // The ESC byte starts a precompiled chunk.
        assert!(Runtime::new("\u{1b}Lua", small()).is_err());
    }

    #[test]
    fn pairs_visits_keys_in_sorted_order() {
        let out = run(
            "function rewrite_commit() local t = { b = 1, a = 2, [3] = 'x', [1] = 'y', [true] = 0 } local out = {} for k, v in pairs(t) do out[#out + 1] = tostring(k) end return { order = table.concat(out, ',') } end",
            json!({}),
        )
        .unwrap();
        assert_eq!(out, json!({"order": "true,1,3,a,b"}));
    }

    #[test]
    fn next_only_tests_for_empty() {
        let out = run(
            "function rewrite_commit() return { empty = next({}) == nil, full = next({ a = 1 }) == 'a' } end",
            json!({}),
        )
        .unwrap();
        assert_eq!(out, json!({"empty": true, "full": true}));
        let e = fails("function rewrite_commit() return next({ a = 1 }, 'a') end");
        assert!(e.contains("next"), "{e}");
    }

    #[test]
    fn tostring_never_prints_an_address() {
        let out = run(
            "function rewrite_commit() return { t = tostring({}), f = tostring(print or rewrite_commit) } end",
            json!({}),
        )
        .unwrap();
        assert_eq!(out, json!({"t": "table", "f": "function"}));
    }

    #[test]
    fn helpers_are_pure() {
        let out = run(
            "function rewrite_commit() return { r = regex_replace('a-1 b-22', '[0-9]+', 'N'), m = regex_match('x@y', '@'), h = stable_hash('abc', 's'), h2 = stable_hash('abc', 's'), x = rand(7), wd = weekday(0), hr = hour(3600 * 5, 120) } end",
            json!({}),
        )
        .unwrap();
        assert_eq!(out["r"], "a-N b-N");
        assert_eq!(out["m"], true);
        assert_eq!(out["h"], out["h2"]);
        assert_eq!(out["x"], json!(rand(7)));
        assert_eq!(out["wd"], 4); // Thursday
        assert_eq!(out["hr"], 7);
    }

    #[test]
    fn trailer_helpers_read_and_edit_messages() {
        let msg = "s\n\nb\n\nSigned-off-by: A <a@x>\nFoo: 1\n";
        let out = run(
            "function rewrite_commit(c) local t = trailers(c.message) return { n = #t, k = t[1].key, v = t[1].value, set = set_trailer(c.message, 'Foo', '2'), add = set_trailer(c.message, 'Foo', '2', 'add'), skip = set_trailer(c.message, 'Foo', '2', 'skip'), rm = remove_trailer(c.message, 'foo'), none = #trailers('just a subject') } end",
            json!({ "message": msg }),
        )
        .unwrap();
        assert_eq!(out["n"], 2);
        assert_eq!(out["k"], "Signed-off-by");
        assert_eq!(out["v"], "A <a@x>");
        assert_eq!(out["set"], "s\n\nb\n\nSigned-off-by: A <a@x>\nFoo: 2\n");
        assert_eq!(out["add"], format!("{msg}Foo: 2\n"));
        assert_eq!(out["skip"], msg);
        assert_eq!(out["rm"], "s\n\nb\n\nSigned-off-by: A <a@x>\n");
        assert_eq!(out["none"], 0);
        let e = fails("function rewrite_commit() return set_trailer('x', 'bad key', 'v') end");
        assert!(e.contains("set_trailer"), "{e}");
        let e =
            fails("function rewrite_commit() return set_trailer('x', 'K', 'v', 'sideways') end");
        assert!(e.contains("replace, add or skip"), "{e}");
    }

    #[test]
    fn at_reads_a_date_in_an_offset() {
        let out = run(
            "function rewrite_commit() return { a = at('1970-01-02'), b = at('2024-03-05 18:30', 120), c = at('2024-02-29T00:00:30', -60), d = at('2000-01-01 00:00') } end",
            json!({}),
        )
        .unwrap();
        assert_eq!(out["a"], 86_400);
        // 2024-03-05 18:30 at +02:00 is 16:30 UTC.
        assert_eq!(out["b"], 1_709_656_200);
        assert_eq!(out["c"], 1_709_164_830 + 3600);
        assert_eq!(out["d"], 946_684_800);
        for bad in [
            "2024-13-01",
            "2023-02-29",
            "2024-01-01 25:00",
            "nonsense",
            "2024-1",
        ] {
            let e = fails(&format!("function rewrite_commit() return at('{bad}') end"));
            assert!(e.contains("at:"), "{bad}: {e}");
        }
    }

    #[test]
    fn a_bad_regex_is_a_script_error() {
        let e = fails("function rewrite_commit() return regex_match('a', '(') end");
        assert!(e.contains("bad regex"), "{e}");
    }

    #[test]
    fn log_lines_come_back_and_are_capped() {
        let rt = Runtime::new(
            "function rewrite_commit() for i = 1, 100 do log('line ' .. i) end end",
            Limits::default(),
        )
        .unwrap();
        let (_, logs) = rt.call("rewrite_commit", &[json!({}), json!({})]).unwrap();
        assert_eq!(logs.len(), MAX_LOG_LINES);
        assert_eq!(logs[0], "line 1");
    }

    #[test]
    fn functions_lists_the_entry_points() {
        let rt = Runtime::new(
            "function rewrite_commit() end function helper() end",
            Limits::default(),
        )
        .unwrap();
        assert_eq!(rt.functions(), vec!["rewrite_commit"]);
    }
}
