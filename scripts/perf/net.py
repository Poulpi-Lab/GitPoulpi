import subprocess, time, os
FIX = r"C:\Projets\_gp_perf\fix"
GIT = r"C:\Program Files\Git\mingw64\bin\git.exe"
def t(name, *a):
    s = time.perf_counter(); r = subprocess.run([GIT, *a], cwd=FIX, capture_output=True); print(f"{name:<55} {(time.perf_counter()-s)*1000:9.1f} ms rc={r.returncode}", flush=True)
for i in range(2):
    t("raw: git fetch origin (nothing new)", "fetch", "origin")
    t("raw: git fetch tag mirror refspec", "fetch", "--prune", "--no-tags", "origin", "+refs/tags/*:refs/gitpoulpi/remote-tags/origin/*")
    t("raw: git for-each-ref refs/remotes/origin", "for-each-ref", "--format=%(refname) %(objectname)", "refs/remotes/origin")
    t("raw: git pull --ff-only (up to date)", "pull", "--ff-only")
    t("raw: git push origin main (up to date)", "push", "origin", "main")
    t("raw: git ls-remote origin", "ls-remote", "origin")
