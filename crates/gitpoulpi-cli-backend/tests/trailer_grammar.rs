//! The trailer grammar of `gitpoulpi_core::domain::rewrite::trailers` against
//! the real thing: `git interpret-trailers --parse` on a fixed list of
//! messages and on a few hundred generated from a pool of lines (a seeded
//! generator, so a failure reproduces).

use std::io::Write;
use std::process::{Command, Stdio};

use gitpoulpi_core::domain::rewrite::trailers;

fn git_parse(message: &str) -> String {
    let mut cmd = Command::new("git");
    cmd.args(["interpret-trailers", "--parse"])
        .env("GIT_CONFIG_NOSYSTEM", "1")
        .env(
            "GIT_CONFIG_GLOBAL",
            if cfg!(windows) { "NUL" } else { "/dev/null" },
        )
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    let mut child = cmd.spawn().expect("spawn git");
    let mut pipe = child.stdin.take().unwrap();
    let input = message.as_bytes().to_vec();
    let writer = std::thread::spawn(move || {
        let _ = pipe.write_all(&input);
    });
    let out = child.wait_with_output().unwrap();
    writer.join().unwrap();
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    String::from_utf8(out.stdout).unwrap()
}

fn ours(message: &str) -> String {
    trailers::parse(message)
        .iter()
        .map(|t| format!("{}: {}\n", t.key, t.value))
        .collect()
}

const FIXED: &[&str] = &[
    "subject\n\nbody\n\nSigned-off-by: A <a@x>\nCo-authored-by: B <b@x>\n",
    "Signed-off-by: A <a@x>\n",
    "Signed-off-by: A <a@x>\n\nbody\n",
    "subject\nSigned-off-by: A <a@x>\n",
    "s\n\nb\n\nFoo: bar\n  baz\n qux\nX: y\n",
    "s\n\nb\n\nFoo: x\n(cherry picked from commit abc)\n",
    "s\n\nb\n\nnot a trailer\nnor this\nnor this\nSigned-off-by: A <a@x>\n",
    "s\n\nb\n\nnot a trailer\nFoo: x\n",
    "s\n\nb\n\nFoo: x\n---\n file | 1 +\nBar: y\n",
    "s\n\nb\n\nFoo: x\n# comment\nBar: y\n",
    "s\n\nb\n\nFoo bar: x\n",
    "s\n\nb\n\nFoo:x\n",
    "s\n\nb\n\nFoo:\nBar: y\n",
    "s\n\nb\n\nFoo=x\nBar: y\n",
    "s\n\nb\n\nFoo: x\n\n\n",
    "s\n\nb\n\nFoo: x",
    "s\n\nb\n\nFoo : x\n",
    "s\n\nb\n\nFoo: x\n   \nBar: y\n",
    "s\n\nb\n\n Foo: x\n",
    "s\n\nb\n\n1Foo: x\n_a: y\n",
    "s\r\n\r\nb\r\n\r\nFoo: x\r\nBar: y\r\n",
];

#[test]
fn trailer_grammar_matches_git_on_the_fixed_cases() {
    for message in FIXED {
        assert_eq!(ours(message), git_parse(message), "message {message:?}");
    }
}

const POOL: &[&str] = &[
    "subject",
    "",
    "",
    "body text here",
    "Signed-off-by: A <a@x>",
    "Signed-off-by:x",
    "Foo: bar",
    "Foo:bar",
    "Foo : spaced",
    "Foo\t: tab",
    "  continued",
    "\tcontinued too",
    "(cherry picked from commit abc123)",
    "# a comment",
    "---",
    "--- patch",
    "Foo bar: nope",
    "Foo=x",
    "not a trailer",
    "Bar:",
    "   ",
    "X-Y-1: z",
    "_k: v",
    ": nokey",
    "Co-authored-by: B <b@x>",
    "Fixes: #12",
];

/// A tiny deterministic generator (no dependency for a test).
struct Lcg(u64);

impl Lcg {
    fn next(&mut self, n: usize) -> usize {
        self.0 = self
            .0
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(1_442_695_040_888_963_407);
        ((self.0 >> 33) as usize) % n
    }
}

#[test]
fn trailer_grammar_matches_git_on_generated_messages() {
    let mut rng = Lcg(17);
    for _ in 0..1500 {
        let lines = 1 + rng.next(9);
        let mut message: String = (0..lines)
            .map(|_| POOL[rng.next(POOL.len())])
            .collect::<Vec<_>>()
            .join("\n");
        if rng.next(4) != 0 {
            message.push('\n');
        }
        assert_eq!(ours(&message), git_parse(&message), "message {message:?}");
    }
}
