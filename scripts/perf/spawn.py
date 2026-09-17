import subprocess, time, os, sys, statistics
FIX = os.path.join(os.path.dirname(os.path.abspath(__file__)), "fix")
GIT_SHIM = r"C:\Program Files\Git\cmd\git.exe"
GIT_REAL = r"C:\Program Files\Git\mingw64\bin\git.exe"
N = int(sys.argv[1]) if len(sys.argv) > 1 else 5
def t(name, args, cwd=FIX):
    ms = []
    for _ in range(N):
        s = time.perf_counter(); subprocess.run(args, cwd=cwd, stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL); ms.append((time.perf_counter() - s) * 1000)
    print(f"{name:<48} min {min(ms):8.1f}  med {statistics.median(ms):8.1f}", flush=True)
t("shim git --version", [GIT_SHIM, "--version"])
t("real git --version", [GIT_REAL, "--version"])
t("shim git status porcelain=v2 -uall", [GIT_SHIM, "status", "--porcelain=v2", "-z", "--untracked-files=all"])
t("real git status porcelain=v2 -uall", [GIT_REAL, "status", "--porcelain=v2", "-z", "--untracked-files=all"])
t("real git add -A (clean tree)", [GIT_REAL, "add", "-A"])
t("real git restore --staged . (clean)", [GIT_REAL, "restore", "--staged", "."])
t("real git log all 5000 date-order", [GIT_REAL, "log", "--branches", "--tags", "--remotes", "--date-order", "-n", "5000", "--format=%H %P %s"])
t("real git log all 200 date-order", [GIT_REAL, "log", "--branches", "--tags", "--remotes", "--date-order", "-n", "200", "--format=%H %P %s"])
t("real git for-each-ref", [GIT_REAL, "for-each-ref"])
