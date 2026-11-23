import subprocess, time, os, statistics, glob, sys
HERE = os.path.dirname(os.path.abspath(__file__))
EXE = os.environ.get("GITPOULPI_EXE") or os.path.join(os.path.dirname(os.path.dirname(HERE)), "target", "release", "gitpoulpi" + (".exe" if os.name == "nt" else ""))
FIX = r"C:\src\_gp_perf\fix"
CFG = os.path.join(HERE, "cfg-cli"); os.makedirs(CFG, exist_ok=True)
env = dict(os.environ, GITPOULPI_CONFIG_DIR=CFG)
def t(name, args, n=5, stdin=None):
    ms = []
    for _ in range(n):
        s = time.perf_counter(); r = subprocess.run([EXE, *args], cwd=FIX, env=env, capture_output=True); ms.append((time.perf_counter() - s) * 1000)
    print(f"{name:<52} min {min(ms):8.1f} med {statistics.median(ms):8.1f} rc={r.returncode} {r.stderr[:120]!r}", flush=True)
t("gitpoulpi --version (process floor)", ["--version"])
t("gitpoulpi status --json (embedded daemon)", ["status", "--json"])
t("gitpoulpi log --limit 50 --json (embedded)", ["log", "--limit", "50", "--json"])
# standalone daemon
d = subprocess.Popen([EXE, "daemon", "run", "--session", "clibench", "--backend", "git2"], cwd=FIX, env=env, stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
time.sleep(3)
t("gitpoulpi -s clibench status --json (running daemon)", ["-s", "clibench", "status", "--json"])
t("gitpoulpi -s clibench log --limit 50 (running daemon)", ["-s", "clibench", "log", "--limit", "50", "--json"])
t("gitpoulpi -s clibench stage --all (running daemon)", ["-s", "clibench", "stage", "--help"], n=1)
subprocess.run([EXE, "-s", "clibench", "daemon", "stop"], cwd=FIX, env=env, capture_output=True)
d.kill()
