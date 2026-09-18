#!/usr/bin/env bash
# Builds the demo repositories the user-guide screenshots are taken on.
# Deterministic: fixed authors, fixed dates, no machine git config.
#
#   fixture.sh <root>     creates <root>/orbit, <root>/orbit.git (its
#                         "origin"), <root>/orbit-conflict (stopped on a
#                         conflicting merge) and <root>/home (a fake $HOME)
#
# Never touches the network: origin is a bare repository next to the others.
set -euo pipefail

root=${1:?usage: fixture.sh <root>}
[ -e "$root" ] && [ -n "$(ls -A "$root" 2>/dev/null)" ] && { echo "fixture: $root is not empty" >&2; exit 1; }
mkdir -p "$root/home/src"
root=$(cd "$root" && pwd)

# Cut off from the machine's git config and from any signing setup.
export HOME="$root/home" GIT_CONFIG_GLOBAL=/dev/null GIT_CONFIG_SYSTEM=/dev/null
export GIT_TERMINAL_PROMPT=0 LC_ALL=C TZ=UTC
git() { command git -c init.defaultBranch=main -c commit.gpgsign=false -c core.autocrlf=false "$@"; }

ALICE="Alice Martin <alice@example.com>"
BRUNO="Bruno Costa <bruno@example.com>"
DANA="Dana Ortiz <dana@example.com>"
CHEN="Chen Wei <chen@example.com>"

# commit "<date>" "<author>" "<message>"  -- stages everything first
commit() {
  local when=$1 who=$2 msg=$3
  git add -A
  GIT_AUTHOR_DATE="$when +0000" GIT_COMMITTER_DATE="$when +0000" \
  GIT_AUTHOR_NAME="${who% <*}" GIT_AUTHOR_EMAIL="${who#*<}" \
  GIT_COMMITTER_NAME="${who% <*}" GIT_COMMITTER_EMAIL="${who#*<}" \
    git commit -q --allow-empty -m "$msg"
}
# merge "<date>" "<author>" <branch>
merge() {
  local when=$1 who=$2 br=$3
  GIT_AUTHOR_DATE="$when +0000" GIT_COMMITTER_DATE="$when +0000" \
  GIT_AUTHOR_NAME="${who% <*}" GIT_AUTHOR_EMAIL="${who#*<}" \
  GIT_COMMITTER_NAME="${who% <*}" GIT_COMMITTER_EMAIL="${who#*<}" \
    git merge -q --no-ff -m "Merge branch '$br'" "$br"
}
tag() { # tag <name> <date>
  GIT_COMMITTER_DATE="$2 +0000" GIT_COMMITTER_NAME=Alice GIT_COMMITTER_EMAIL=alice@example.com \
    git tag -a "$1" -m "$1"
}
put() { mkdir -p "$(dirname "$1")"; cat > "$1"; }

# ---------------------------------------------------------------- orbit
mkdir "$root/orbit" && cd "$root/orbit"
git init -q
git config user.name "Alice Martin"; git config user.email alice@example.com

put README.md <<'X'
# Orbit

A tiny solar-system simulator that draws planets in ASCII.
X
commit "2026-09-02 05:05:00" "$ALICE" "chore: initial commit"
put docs/run.md <<'X'
# Running

    cargo run
X
commit "2026-09-02 17:22:00" "$ALICE" "docs: how to run"
tag v0.1.0 "2026-09-02 17:30:00"

put src/planet.rs <<'X'
//! Planets.

pub struct Planet {
    pub name: &'static str,
    pub radius: f32,
}

pub fn count() -> usize {
    8
}
X
commit "2026-09-03 05:38:00" "$BRUNO" "feat(planet): count the planets"
put src/orbit.rs <<'X'
pub fn period(a: f32) -> f32 { a.powf(1.5) }
X
cat >> src/planet.rs <<'X'

/// Whether the planet is a gas giant.
pub fn is_giant(p: &Planet) -> bool {
    p.radius > 20.0
}
X
commit "2026-09-03 17:54:00" "$BRUNO" "feat(render): show the orbits"

git switch -q -c fix/planet-radius
put src/planet.rs <<'X'
//! Planets.

pub struct Planet {
    pub name: &'static str,
    pub radius: f32,
}

pub fn count() -> usize {
    8
}

pub fn radius(p: &Planet) -> f32 {
    p.radius.max(0.1)
}

/// Whether the planet is a gas giant.
pub fn is_giant(p: &Planet) -> bool {
    p.radius > 20.0
}
X
commit "2026-09-04 06:10:00" "$ALICE" "feat(planet): orbit radius"
git switch -q main
put src/render.rs <<'X'
pub fn draw(rows: usize) -> String { "o".repeat(rows) }
X
commit "2026-09-04 18:27:00" "$CHEN" "fix(planet): use the radius table"
merge "2026-09-05 06:43:00" "$ALICE" fix/planet-radius
git branch -d fix/planet-radius -q

put docs/earth.md <<'X'
# A year on Earth
X
commit "2026-09-05 18:59:00" "$DANA" "test: a year on Earth"
tag v0.2.0 "2026-09-05 19:10:00"

git switch -q -c feature/ascii-art
put src/art.rs <<'X'
pub const SUN: &str = "(O)";
X
commit "2026-09-06 19:32:00" "$DANA" "feat(art): draw the sun"
put src/art.rs <<'X'
pub const SUN: &str = "(O)";
pub const MOON: &str = ")";
X
commit "2026-09-07 07:48:00" "$DANA" "feat(art): add a ringed planet"
git switch -q main
git switch -q -c feature/moons
put src/moons.rs <<'X'
pub const MOONS: [(&str, u8); 3] = [("Moon", 1), ("Phobos", 2), ("Io", 3)];
X
commit "2026-09-06 07:16:00" "$BRUNO" "feat(planet): total moons"
git switch -q main
put docs/plan.md <<'X'
# Plan
- moons
- ASCII art
X
commit "2026-09-07 20:04:00" "$ALICE" "docs: list the planned work"
merge "2026-09-08 08:21:00" "$BRUNO" feature/moons 2>/dev/null || true
git switch -q -c release/0.3
put VERSION <<'X'
0.3.0
X
commit "2026-09-08 20:37:00" "$CHEN" "chore: bump the version"
tag v0.3.0 "2026-09-08 20:45:00"
git switch -q main
put Cargo.lock <<'X'
# Cargo.lock: nothing to see
X
commit "2026-09-09 06:30:00" "$CHEN" "chore: update the lockfile"
put src/render.rs <<'X'
pub fn draw(rows: usize) -> String { "o".repeat(rows.min(80)) }
X
commit "2026-09-09 08:53:00" "$CHEN" "fix(render): widen the viewport"
put src/render.rs <<'X'
//! Rendering.
pub fn draw(rows: usize) -> String { "o".repeat(rows.min(80)) }
X
commit "2026-09-09 14:12:00" "$BRUNO" "chore: tidy the render module"
put src/table.rs <<'X'
//! The table of planets.

/// Names, innermost first.
pub fn names() -> Vec<&'static str> {
    vec!["Mercury", "Venus", "Earth", "Mars"]
}

/// How many planets the table holds.
pub fn count() -> usize {
    names().len()
}

/// The planet at `index`, if any.
pub fn name(index: usize) -> Option<&'static str> {
    names().get(index).copied()
}

/// Whether the table is empty.
pub fn is_empty() -> bool {
    names().is_empty()
}
X
commit "2026-09-09 21:09:00" "$ALICE" "feat(render): table of planets"
put docs/design.md <<'X'
# Design notes

The renderer draws one ring per orbit.
X
commit "2026-09-10 21:42:00" "$DANA" "docs: design notes"

# --------------------------------------------------- origin + remote refs
git clone -q --bare "$root/orbit" "$root/orbit.git"
git remote add origin "$root/orbit.git"
git fetch -q origin
git branch -q --set-upstream-to=origin/main main
git branch -q -D feature/moons 2>/dev/null || true
git fetch -q origin 'refs/heads/*:refs/remotes/origin/*'
git switch -q -c feature/moons origin/feature/moons 2>/dev/null || git branch -q feature/moons origin/feature/moons
git switch -q main

# ----------------------------------------------- stash + working changes
put README.md <<'X'
# Orbit

A tiny solar-system simulator that draws planets in ASCII.

Stash me: work in progress on the ellipse maths.
X
GIT_COMMITTER_DATE="2026-10-01 16:49:00 +0000" GIT_AUTHOR_DATE="2026-10-01 16:49:00 +0000" \
  GIT_COMMITTER_NAME=Alice GIT_COMMITTER_EMAIL=alice@example.com git stash push -q -m "WIP ellipti"
put README.md <<'X'
# Orbit

A tiny solar-system simulator that draws planets in ASCII.

## Status

Eight planets, three moons, one very large sun.
X
put docs/design.md <<'X'
# Design notes

The renderer draws one ring per orbit.
Moons are drawn after their planet.
X
put src/planet.rs <<'X'
//! Planets.

pub struct Planet {
    pub name: &'static str,
    pub radius: f32,
}

pub fn count() -> usize {
    9
}

pub fn radius(p: &Planet) -> f32 {
    p.radius.max(0.1)
}

/// Whether the planet is a gas giant.
pub fn is_giant(p: &Planet) -> bool {
    p.radius > 20.0
}
X
put src/table.rs <<'X'
//! The table of planets, with the gas giants.

/// Names, innermost first.
pub fn names() -> Vec<&'static str> {
    vec!["Mercury", "Venus", "Earth", "Mars"]
}

/// How many planets the table holds.
pub fn count() -> usize {
    names().len()
}

/// The planet at `index`, if any.
pub fn name(index: usize) -> Option<&'static str> {
    names().get(index).copied()
}

/// Whether the table is empty (it never is).
pub fn is_empty() -> bool {
    false
}
X
put src/notes.rs <<'X'
// scratch notes for the gas giants
X
git add src/notes.rs

# ------------------------------------------------- orbit-conflict (merge)
mkdir "$root/orbit-conflict" && cd "$root/orbit-conflict"
git init -q
git config user.name "Alice Martin"; git config user.email alice@example.com
put src/orbit.rs <<'X'
//! Orbital maths.

/// Period of a circular orbit.
pub fn period(a: f32) -> f32 {
    a.powf(1.5)
}

/// Speed on a circular orbit.
pub fn speed(a: f32) -> f32 {
    1.0 / a.sqrt()
}

/// Name of the central body.
pub fn center() -> &'static str {
    "Sun"
}
X
mkdir -p assets && printf 'PNG\0\1\2\3 base' > assets/logo.bin
commit "2026-09-02 05:05:00" "$ALICE" "chore: initial commit"
git switch -q -c feature/kepler
printf 'PNG\0\1\2\3 kepler' > assets/logo.bin
put src/orbit.rs <<'X'
//! Orbital maths.

/// Period of a circular orbit, in Earth years.
pub fn period(a: f32) -> f32 {
    a * a.sqrt()
}

/// Speed on a circular orbit.
pub fn speed(a: f32) -> f32 {
    (1.0 / a).sqrt()
}

/// Name of the central body.
pub fn center() -> &'static str {
    "Sun"
}
X
commit "2026-09-03 10:00:00" "$BRUNO" "feat(orbit): Kepler's third law"
git switch -q main
printf 'PNG\0\1\2\3 days' > assets/logo.bin
put src/orbit.rs <<'X'
//! Orbital maths.

/// Period of a circular orbit, in days.
pub fn period(a: f32) -> f32 {
    365.25 * a.powf(1.5)
}

/// Speed on a circular orbit.
pub fn speed(a: f32) -> f32 {
    1.0 / a.sqrt().max(0.01)
}

/// Name of the central body.
pub fn center() -> &'static str {
    "Sol"
}
X
commit "2026-09-03 11:00:00" "$DANA" "feat(orbit): periods in days"
git merge -q feature/kepler >/dev/null 2>&1 || true

# --------------------------------------- a canned "AI" for the screenshots
cat > "$root/fake-ai.sh" <<'X'
#!/usr/bin/env bash
# A local agent that never leaves the machine: reads the prompt on stdin and
# answers with canned text matching the action.
p=$(cat)
case $p in
  *"You review a pull request"*)
    cat <<J
{"summary": "Adds the planet table and a moons helper. The helper builds a new Vec on every call.", "suggestions": [{"path": "src/planets.rs", "line": 12, "side": "new", "severity": "warning", "body": "moons() allocates on every call; return a static slice or build it once."}, {"path": "src/planets.rs", "line": 13, "side": "new", "severity": "info", "body": "count() could read the table's length directly."}]}
J
    ;;
  *"JSON array only"*)
    cat <<J
[{"message":"docs: describe the status of the simulator","files":["README.md","docs/design.md"]},
 {"message":"feat(planet): count nine planets","files":["src/planet.rs"]},
 {"message":"feat(render): document the planet table","files":["src/table.rs"]},
 {"message":"chore: add notes on the gas giants","files":["src/notes.rs"]}]
J
    ;;
  *"explain code changes"*)
    cat <<J
The README gains a Status section, the design notes mention that moons are
drawn after their planet, the planet count goes from 8 to 9, and the planet
table gets doc comments.

Worth double-checking:
- count() now says 9: is the ninth planet really in the table?
- notes.rs is new and still empty of code.
J
    ;;
  *"git commit messages"*)
    printf '%s\n\n%s\n' "docs: add a status section and document the planet table" \
      "Describe what the simulator draws, note that moons follow their planet and document the table functions."
    ;;
  *) echo "ok" ;;
esac
X
chmod +x "$root/fake-ai.sh"

echo "fixture ready: $root"
