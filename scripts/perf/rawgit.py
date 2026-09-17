import subprocess, time, os, glob, sys
HERE = os.path.dirname(os.path.abspath(__file__)); FIX = sys.argv[1] if len(sys.argv) > 1 else os.path.join(HERE, "fix")
GIT = r"C:\Program Files\Git\mingw64\bin\git.exe"
def t(name, *a):
    s = time.perf_counter(); r = subprocess.run([GIT, *a], cwd=FIX, capture_output=True); ms = (time.perf_counter() - s) * 1000
    print(f"{name:<50} {ms:9.1f} ms  out {len(r.stdout)//1024}KB rc={r.returncode}", flush=True)
def touch(n):
    for f in sorted(glob.glob(os.path.join(FIX, "src", "*", "*.txt")))[:n]:
        with open(f, "a", newline="\n") as fh: fh.write("perf edit\n")
print(subprocess.run([GIT, "config", "--show-origin", "--get-regexp", "core\\..*"], cwd=FIX, capture_output=True, text=True).stdout)
for n in (200, 2000):
    touch(n)
    t(f"[{n}] status porcelain", "status", "--porcelain=v2", "-z", "--untracked-files=all")
    t(f"[{n}] diff (wt vs index) -U3", "diff", "--no-color", "--no-ext-diff", "-U3")
    t(f"[{n}] add -A", "add", "-A")
    t(f"[{n}] diff --cached", "diff", "--cached", "--no-color", "-U3")
    t(f"[{n}] restore --staged .", "restore", "--staged", ".")
    t(f"[{n}] reset -q (mixed, all)", "reset", "-q")
    t(f"[{n}] add -A (2nd, objects exist)", "add", "-A")
    t(f"[{n}] reset -q -- <one path>", "reset", "-q", "--", "src/mod00/file0000.txt")
    t(f"[{n}] checkout -q -- <one path>", "checkout", "-q", "--", "src/mod00/file0000.txt")
    t(f"[{n}] reset -q (all)", "reset", "-q")
    t(f"[{n}] checkout -- . (discard all)", "checkout", "-q", "--", ".")
t("count-objects", "count-objects", "-v")
