//! The JSON contract between the engine and a commit script
//! (the history-rewrite design, "The JSON contract").
//!
//! The request is built as one JSON value, the shape an external command
//! reads as a line (S11) and the embedded Lua runtime gets as two tables
//! (`c` = `request.commit`; `ctx` = the rest). The response is read with
//! the plan's rules: a field absent means "keep", unknown fields are an
//! error, and `parents` / `tree` belong to the engine.

use gitpoulpi_core::domain::rewrite::MAX_OFFSET_MINUTES;
use gitpoulpi_core::domain::Oid;
use serde_json::{json, Map, Value};

use super::super::objects::Person;

/// The contract's version, in every request.
pub(super) const VERSION: u32 = 1;

pub(super) fn person_json(p: &Person) -> Value {
    json!({
        "name": p.name,
        "email": p.email,
        "time": p.seconds,
        "offset_minutes": p.offset,
    })
}

/// A neighbour in the window: no message body, to keep requests light.
pub(super) struct Summary {
    pub(super) id: Oid,
    pub(super) author: Option<Person>,
    pub(super) committer: Option<Person>,
    pub(super) subject: String,
}

impl Summary {
    fn json(&self) -> Value {
        let mut m = Map::new();
        m.insert("id".into(), json!(self.id.to_hex()));
        if let Some(a) = &self.author {
            m.insert("author".into(), person_json(a));
        }
        if let Some(c) = &self.committer {
            m.insert("committer".into(), person_json(c));
        }
        m.insert("subject".into(), json!(self.subject));
        Value::Object(m)
    }
}

/// The commit as the script sees it (the original object, with the people
/// and the message as the rules before this one left them).
pub(super) struct CommitView {
    pub(super) id: Oid,
    pub(super) parents: Vec<Oid>,
    pub(super) tree: Option<Oid>,
    pub(super) author: Option<Person>,
    pub(super) committer: Option<Person>,
    pub(super) message: String,
    pub(super) signed: bool,
}

pub(super) struct Request<'a> {
    pub(super) index: u64,
    pub(super) total: u64,
    pub(super) commit: &'a CommitView,
    pub(super) before: &'a [Summary],
    pub(super) after: &'a [Summary],
    pub(super) params: &'a Value,
    pub(super) state: &'a Value,
}

impl Request<'_> {
    pub(super) fn json(&self) -> Value {
        let c = self.commit;
        let mut commit = Map::new();
        commit.insert("id".into(), json!(c.id.to_hex()));
        commit.insert(
            "parents".into(),
            Value::Array(c.parents.iter().map(|p| json!(p.to_hex())).collect()),
        );
        if let Some(t) = c.tree {
            commit.insert("tree".into(), json!(t.to_hex()));
        }
        if let Some(a) = &c.author {
            commit.insert("author".into(), person_json(a));
        }
        if let Some(a) = &c.committer {
            commit.insert("committer".into(), person_json(a));
        }
        commit.insert("message".into(), json!(c.message));
        commit.insert("signed".into(), json!(c.signed));
        json!({
            "version": VERSION,
            "kind": "commit",
            "index": self.index,
            "total": self.total,
            "commit": commit,
            "window": {
                "before": self.before.iter().map(Summary::json).collect::<Vec<_>>(),
                "after": self.after.iter().map(Summary::json).collect::<Vec<_>>(),
            },
            "params": self.params,
            "state": self.state,
        })
    }

    /// What the Lua function receives: `(c, ctx)`.
    pub(super) fn lua_args(&self) -> [Value; 2] {
        let mut whole = self.json();
        let commit = whole["commit"].take();
        let ctx = json!({
            "index": whole["index"],
            "total": whole["total"],
            "window": whole["window"],
            "params": whole["params"],
            "state": whole["state"],
        });
        [commit, ctx]
    }
}

/// A partial person: what the script wants to change.
#[derive(Debug, Default, PartialEq)]
pub(super) struct PersonPatch {
    pub(super) name: Option<String>,
    pub(super) email: Option<String>,
    pub(super) time: Option<i64>,
    pub(super) offset_minutes: Option<i16>,
}

#[derive(Debug, Default, PartialEq)]
pub(super) struct Patch {
    pub(super) author: Option<PersonPatch>,
    pub(super) committer: Option<PersonPatch>,
    pub(super) message: Option<String>,
    /// Replaces the carried state.
    pub(super) state: Option<Value>,
    pub(super) log: Option<String>,
}

fn integer(v: &Value, what: &str) -> Result<i64, String> {
    if let Some(i) = v.as_i64() {
        return Ok(i);
    }
    match v.as_f64() {
        Some(f) if f.fract() == 0.0 && f.abs() < 9.0e15 => Ok(f as i64),
        _ => Err(format!("`{what}` must be a whole number")),
    }
}

fn text(v: &Value, what: &str) -> Result<String, String> {
    v.as_str()
        .map(str::to_string)
        .ok_or_else(|| format!("`{what}` must be text"))
}

fn ident_text(v: &Value, what: &str) -> Result<String, String> {
    let s = text(v, what)?;
    if s.contains(['\n', '<', '>', '\0']) {
        return Err(format!(
            "`{what}` may not contain a newline, `<`, `>` or NUL"
        ));
    }
    Ok(s)
}

fn person_patch(v: &Value, who: &str) -> Result<PersonPatch, String> {
    let Some(obj) = v.as_object() else {
        return Err(format!("`{who}` must be a table of fields"));
    };
    let mut p = PersonPatch::default();
    for (k, v) in obj {
        match k.as_str() {
            "name" => p.name = Some(ident_text(v, &format!("{who}.name"))?),
            "email" => p.email = Some(ident_text(v, &format!("{who}.email"))?),
            "time" => p.time = Some(integer(v, &format!("{who}.time"))?),
            "offset_minutes" => {
                let o = integer(v, &format!("{who}.offset_minutes"))?;
                if o.abs() > i64::from(MAX_OFFSET_MINUTES) {
                    return Err(format!("`{who}.offset_minutes` is out of range"));
                }
                p.offset_minutes = Some(o as i16);
            }
            other => return Err(format!("unknown field `{who}.{other}`")),
        }
    }
    Ok(p)
}

/// Reads a script's answer. `Null` (a function that returned nothing) is
/// "no change". The error text names the field.
pub(super) fn parse_response(v: &Value) -> Result<Patch, String> {
    let mut patch = Patch::default();
    let obj = match v {
        Value::Null => return Ok(patch),
        Value::Object(o) => o,
        _ => return Err("the result must be a table of fields (or nothing)".into()),
    };
    for (k, v) in obj {
        match k.as_str() {
            "author" => patch.author = Some(person_patch(v, "author")?),
            "committer" => patch.committer = Some(person_patch(v, "committer")?),
            "message" => {
                let m = text(v, "message")?;
                if m.contains('\0') {
                    return Err("`message` may not contain NUL".into());
                }
                patch.message = Some(m);
            }
            "state" => patch.state = Some(v.clone()),
            "log" => patch.log = Some(text(v, "log")?),
            "parents" | "tree" => {
                return Err(format!(
                    "`{k}` belongs to the engine; a script cannot change it"
                ))
            }
            other => return Err(format!("unknown field `{other}`")),
        }
    }
    Ok(patch)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn person(name: &str, time: i64) -> Person {
        Person::parse(format!("{name} <{name}@example.org> {time} +0200").as_bytes()).unwrap()
    }

    #[test]
    fn the_request_has_the_plans_shape() {
        let commit = CommitView {
            id: Oid([1; 20]),
            parents: vec![Oid([2; 20])],
            tree: Some(Oid([3; 20])),
            author: Some(person("alex", 1_760_000_000)),
            committer: Some(person("alex", 1_760_000_300)),
            message: "feat: parser\n".into(),
            signed: true,
        };
        let before = [Summary {
            id: Oid([2; 20]),
            author: Some(person("sam", 5)),
            committer: None,
            subject: "init".into(),
        }];
        let params = json!({"old_email": "a@b"});
        let state = json!({"count": 1});
        let request = Request {
            index: 41,
            total: 1200,
            commit: &commit,
            before: &before,
            after: &[],
            params: &params,
            state: &state,
        };
        let got = request.json();
        assert_eq!(
            got,
            json!({
                "version": 1, "kind": "commit", "index": 41, "total": 1200,
                "commit": {
                    "id": "01".repeat(20), "parents": ["02".repeat(20)],
                    "tree": "03".repeat(20),
                    "author": {"name": "alex", "email": "alex@example.org",
                               "time": 1_760_000_000, "offset_minutes": 120},
                    "committer": {"name": "alex", "email": "alex@example.org",
                                  "time": 1_760_000_300, "offset_minutes": 120},
                    "message": "feat: parser\n", "signed": true,
                },
                "window": {
                    "before": [{"id": "02".repeat(20),
                                "author": {"name": "sam", "email": "sam@example.org",
                                           "time": 5, "offset_minutes": 120},
                                "subject": "init"}],
                    "after": [],
                },
                "params": {"old_email": "a@b"},
                "state": {"count": 1},
            })
        );
        let [c, ctx] = request.lua_args();
        assert_eq!(c["message"], "feat: parser\n");
        assert_eq!(ctx["index"], 41);
        assert_eq!(ctx["state"], json!({"count": 1}));
        assert!(ctx.get("commit").is_none());
    }

    #[test]
    fn a_response_is_read_field_by_field() {
        let patch = parse_response(&json!({
            "author": {"email": "n@example.org", "time": 7.0},
            "message": "m\n", "state": {"count": 42}, "log": "mapped",
        }))
        .unwrap();
        assert_eq!(
            patch.author,
            Some(PersonPatch {
                email: Some("n@example.org".into()),
                time: Some(7),
                ..PersonPatch::default()
            })
        );
        assert_eq!(patch.message.as_deref(), Some("m\n"));
        assert_eq!(patch.state, Some(json!({"count": 42})));
        assert_eq!(parse_response(&Value::Null).unwrap(), Patch::default());
    }

    #[test]
    fn a_bad_response_names_the_field() {
        for (v, needle) in [
            (json!({"tree": "x"}), "`tree` belongs to the engine"),
            (json!({"parents": []}), "`parents` belongs to the engine"),
            (json!({"colour": 1}), "unknown field `colour`"),
            (
                json!({"author": {"nick": "x"}}),
                "unknown field `author.nick`",
            ),
            (json!({"author": {"email": "a<b>"}}), "author.email"),
            (json!({"author": {"time": 1.5}}), "author.time"),
            (
                json!({"committer": {"offset_minutes": 99999}}),
                "out of range",
            ),
            (json!({"message": 3}), "`message` must be text"),
            (json!([1, 2]), "table of fields"),
        ] {
            let e = parse_response(&v).unwrap_err();
            assert!(e.contains(needle), "{v}: {e}");
        }
    }
}
