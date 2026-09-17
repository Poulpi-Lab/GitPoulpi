"""IPC-level bench for the GitPoulpi daemon (scripts/perf; see the perf design).

Starts a standalone daemon (isolated config dir, GITPOULPI_TIMING on, TEMP redirected
so the timing log lands here), then times every operation as the TUI would issue it:
  op RTT                      -- the request alone
  op+refresh ("cycle")        -- request, wait for RepoChanged, then the TUI's
                                 9-query refresh + working-diff reload (what the user waits for)
Usage: python bench.py <git2|system> <repo> [label]
"""
import json, os, socket, struct, subprocess, sys, threading, time, statistics, glob, shutil, queue

HERE = os.path.dirname(os.path.abspath(__file__))
EXE = os.environ.get("GITPOULPI_EXE") or os.path.join(os.path.dirname(os.path.dirname(HERE)), "target", "release", "gitpoulpi" + (".exe" if os.name == "nt" else ""))
BACKEND = sys.argv[1]
REPO = os.path.abspath(sys.argv[2])
LABEL = sys.argv[3] if len(sys.argv) > 3 else os.path.basename(REPO)
REPO_ID = ("\\\\?\\" if os.name == "nt" else "") + os.path.realpath(REPO)
SESSION = f"perf-{BACKEND}"
CFG = os.path.join(HERE, "cfg"); TMP = os.path.join(HERE, "tmp-" + BACKEND)
os.makedirs(CFG, exist_ok=True); os.makedirs(TMP, exist_ok=True)
LOG = os.path.join(TMP, "gitpoulpi-timing.log")
if os.path.exists(LOG): os.remove(LOG)
for f in glob.glob(os.path.join(CFG, "**", SESSION + ".*"), recursive=True): os.remove(f)

env = dict(os.environ, GITPOULPI_CONFIG_DIR=CFG, GITPOULPI_TIMING="1", TEMP=TMP, TMP=TMP)
daemon = subprocess.Popen([EXE, "daemon", "run", "--session", SESSION, "--backend", BACKEND],
                          env=env, cwd=REPO, stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
port = None
for _ in range(200):
    hits = glob.glob(os.path.join(CFG, "**", SESSION + ".port"), recursive=True)
    if hits:
        try: port = int(open(hits[0]).read().strip()); break
        except ValueError: pass
    time.sleep(0.05)
assert port, "daemon did not publish a port"

class Conn:
    def __init__(self):
        self.s = socket.create_connection(("127.0.0.1", port)); self.s.setsockopt(socket.IPPROTO_TCP, socket.TCP_NODELAY, 1)
        self.id = 0; self.lock = threading.Lock(); self.waiters = {}; self.events = queue.Queue()
        threading.Thread(target=self.reader, daemon=True).start()
    def recvn(self, n):
        b = bytearray()
        while len(b) < n:
            c = self.s.recv(n - len(b))
            if not c: raise EOFError
            b += c
        return bytes(b)
    def reader(self):
        try:
            while True:
                n = struct.unpack(">I", self.recvn(4))[0]; raw = self.recvn(n); msg = json.loads(raw)
                if "Response" in msg:
                    r = msg["Response"]; q = self.waiters.pop(r["request_id"], None)
                    if q: q.put((r, n))
                else:
                    self.events.put((time.perf_counter(), msg["Event"]))
        except EOFError: pass
    def send(self, cmd, args=None):
        with self.lock:
            self.id += 1; rid = self.id; q = queue.Queue(); self.waiters[rid] = q
            payload = json.dumps({"id": rid, "session": SESSION, "repo": REPO_ID, "command": cmd, "args": args or {}}).encode()
            self.s.sendall(struct.pack(">I", len(payload)) + payload)
        return q
    def call(self, cmd, args=None):
        r, n = self.send(cmd, args).get(timeout=600)
        if "Err" in r["result"]: raise RuntimeError(f"{cmd}: {r['result']['Err']}")
        return r["result"]["Ok"], n

c = Conn()
LOG_LIMIT = 5000
REFRESH = [("Status", {}), ("Log", {"opts": {"limit": LOG_LIMIT, "skip": 0, "start_ref": None, "all_refs": True, "paths": []}}),
           ("Refs", {}), ("Stashes", {}), ("Worktrees", {}), ("Conflicts", {}), ("Remotes", {}), ("SshCommand", {}), ("RebaseState", {}),
           ("LfsStatus", {})]
WORKDIFF = [("Diff", {"target": "WorkingTreeVsIndex"}), ("Diff", {"target": "IndexVsHead"}), ("Status", {})]

def parallel(reqs):
    t0 = time.perf_counter(); qs = [(cmd, c.send(cmd, a)) for cmd, a in reqs]; out = {}
    for cmd, q in qs:
        r, n = q.get(timeout=600); out.setdefault(cmd, []).append(((time.perf_counter() - t0) * 1000, n))
    return (time.perf_counter() - t0) * 1000, out

def drain():
    while not c.events.empty(): c.events.get_nowait()

rows = []
def op(name, cmd, args=None, cycle=True, n=1):
    """Time `cmd`; with cycle, also the refresh the TUI runs on the RepoChanged it triggers."""
    drain(); t0 = time.perf_counter(); res, size = c.call(cmd, args); t_op = (time.perf_counter() - t0) * 1000
    t_ref = t_wd = 0.0; detail = ""
    if cycle:
        try:
            while True:
                ts, ev = c.events.get(timeout=5)
                if "RepoChanged" in ev: break
        except queue.Empty: pass
        t_ref, out = parallel(REFRESH)
        slow = sorted(((v[-1][0], k, v[-1][1]) for k, v in out.items()), reverse=True)[:3]
        detail = ", ".join(f"{k} {ms:.0f}ms/{sz//1024}KB" for ms, k, sz in slow)
        t_wd, _ = parallel(WORKDIFF)
    total = t_op + t_ref + t_wd
    rows.append((name, t_op, t_ref, t_wd, total, size, detail))
    print(f"{name:<34} op {t_op:8.1f}  refresh {t_ref:8.1f}  workdiff {t_wd:7.1f}  TOTAL {total:8.1f}  resp {size//1024}KB  [{detail}]", flush=True)
    return res

def git(*a): subprocess.run(["git", *a], cwd=REPO, check=True, stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)

def touch(n):
    files = sorted(glob.glob(os.path.join(REPO, "src", "*", "*.txt")))[:n]
    for f in files:
        with open(f, "a", newline="\n") as fh: fh.write("perf edit\n")
    return [os.path.relpath(f, REPO).replace("\\", "/") for f in files]

try:
    # warm
    c.call("Status"); c.call("Hello")
    print(f"== {LABEL} backend={BACKEND}")
    for i in range(3):
        t, out = parallel(REFRESH)
        print(f"refresh (idle) #{i}: {t:.1f}ms  " + ", ".join(f"{k} {v[-1][0]:.0f}ms/{v[-1][1]//1024}KB" for k, v in sorted(out.items(), key=lambda kv: -kv[1][-1][0])[:5]), flush=True)
    for name, cmd, a in [("status (clean)", "Status", {}), ("log 5000 all_refs", "Log", REFRESH[1][1]),
                         ("log first page 200", "Log", {"opts": {"limit": 200, "skip": 0, "start_ref": None, "all_refs": True, "paths": []}}),
                         ("log page skip=4000 lim=500", "Log", {"opts": {"limit": 500, "skip": 4000, "start_ref": None, "all_refs": True, "paths": []}}),
                         ("refs", "Refs", {}), ("conflicts", "Conflicts", {}), ("rebase_state", "RebaseState", {}), ("lfs_status", "LfsStatus", {}),
                         ("stashes", "Stashes", {}), ("worktrees", "Worktrees", {}), ("remotes", "Remotes", {})]:
        ms = []
        for _ in range(3):
            t0 = time.perf_counter(); _, sz = c.call(cmd, a); ms.append((time.perf_counter() - t0) * 1000)
        print(f"read {name:<30} min {min(ms):8.1f} med {statistics.median(ms):8.1f}  resp {sz//1024}KB", flush=True)
    if "--reads-only" in sys.argv: raise SystemExit
    if "--tui-sim" in sys.argv:
        # Emulates daemon_link's select loop: a refresh every 2 s awaited inline;
        # an action is only picked up between refreshes; RepoChanged triggers another refresh.
        import random
        actions = queue.Queue(); done = queue.Queue()
        def loop():
            next_tick = time.perf_counter()
            while True:
                now = time.perf_counter()
                try:
                    item = actions.get(timeout=max(0, next_tick - now))
                except queue.Empty:
                    item = None
                if item is None:
                    parallel(REFRESH); next_tick = time.perf_counter() + 2.0; continue
                if item == "stop": return
                clicked, cmd, a = item
                picked = time.perf_counter()
                c.call(cmd, a); acted = time.perf_counter()
                parallel(REFRESH); parallel(WORKDIFF)
                done.put((clicked, picked, acted, time.perf_counter()))
        th = threading.Thread(target=loop, daemon=True); th.start()
        random.seed(2); res = []
        p = touch(5)
        for i in range(8):
            time.sleep(random.uniform(0.5, 2.5))
            cmd = ("StageAll", {"target": "All"}) if i % 2 == 0 else ("UnstageAll", {"target": "All"})
            actions.put((time.perf_counter(), *cmd))
            clicked, picked, acted, painted = done.get()
            res.append(((picked-clicked)*1000, (acted-picked)*1000, (painted-acted)*1000, (painted-clicked)*1000))
            print(f"tui-sim {cmd[0]:<11} queued {res[-1][0]:7.1f}  op {res[-1][1]:7.1f}  refresh+workdiff {res[-1][2]:7.1f}  click->data {res[-1][3]:7.1f}", flush=True)
        actions.put("stop"); th.join()
        print("tui-sim median click->data: %.1f ms" % statistics.median(r[3] for r in res))
        git("reset", "-q"); git("checkout", "-q", "--", ".")
        raise SystemExit
    # staging scenarios
    for n in [int(x) for x in os.environ.get("SIZES", "1,200").split(",")]:
        paths = touch(n)
        t0 = time.perf_counter(); st, sz = c.call("Status"); print(f"read status with {n} modified: {(time.perf_counter()-t0)*1000:.1f}ms ({len(st['files'])} entries, {sz//1024}KB)")
        t0 = time.perf_counter(); _, sz = c.call("Diff", {"target": "WorkingTreeVsIndex"}); print(f"read diff wt with {n} modified: {(time.perf_counter()-t0)*1000:.1f}ms ({sz//1024}KB)")
        op(f"stage file ({n} dirty)", "StageFile", {"target": {"File": paths[0]}})
        op(f"unstage file ({n} dirty)", "UnstageFile", {"target": {"File": paths[0]}})
        op(f"stage all ({n} dirty)", "StageAll", {"target": "All"})
        op(f"unstage all ({n} dirty)", "UnstageAll", {"target": "All"})
        op(f"discard 1 file ({n} dirty)", "Discard", {"paths": [paths[0]]})
        if n == 200:
            op(f"discard 50 files", "Discard", {"paths": paths[1:51]})
        git("checkout", "-q", "--", ".")
    paths = touch(3)
    c.call("StageAll", {"target": "All"})
    op("commit (3 files)", "Commit", {"message": "perf commit", "opts": {"amend": False, "sign": False}})
    paths = touch(20)
    op("stash push (20 files)", "StashPush", {"message": "perf", "include_untracked": True})
    op("stash pop", "StashPop", {"id": 0})
    git("checkout", "-q", "--", ".")
    head = c.call("Resolve", {"rev": "HEAD"})[0]
    op("branch create", "BranchCreate", {"name": "perf/b1", "at": head})
    op("checkout branch", "Checkout", {"target": {"Branch": "perf/b1"}})
    op("checkout main", "Checkout", {"target": {"Branch": "main"}})
    op("branch delete", "BranchDelete", {"name": "perf/b1"})
    # big file diff
    big = os.path.join(REPO, "big.txt")
    if os.path.exists(big):
        data = open(big).read()
        with open(big, "w", newline="\n") as fh: fh.write(data.replace("x", "y"))
        t0 = time.perf_counter(); _, sz = c.call("Diff", {"target": "WorkingTreeVsIndex"}); print(f"read diff big.txt (20k lines rewritten): {(time.perf_counter()-t0)*1000:.1f}ms ({sz//1024}KB)")
        t0 = time.perf_counter(); _, sz = c.call("DiffFiles", {"target": "WorkingTreeVsIndex"}); print(f"read diff_files big.txt: {(time.perf_counter()-t0)*1000:.1f}ms ({sz//1024}KB)")
        git("checkout", "-q", "--", "big.txt")
    # network against the local bare remote
    if "--net" in sys.argv:
        op("fetch origin", "Fetch", {"remote": "origin"})
        touch(2); c.call("StageAll", {"target": "All"}); c.call("Commit", {"message": "to push", "opts": {"amend": False, "sign": False}})
        op("push origin", "Push", {"remote": "origin"})
        op("pull (up to date)", "Pull", {})
finally:
    try: c.call("Shutdown")
    except Exception: pass
    time.sleep(0.3); daemon.kill()
    shutil.copy(LOG, os.path.join(HERE, f"timing-{LABEL}-{BACKEND}.log")) if os.path.exists(LOG) else None
