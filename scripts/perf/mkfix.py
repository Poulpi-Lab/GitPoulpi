import random, subprocess, sys, os, shutil
random.seed(1)
HERE = sys.argv[1] if len(sys.argv) > 1 else os.path.dirname(os.path.abspath(__file__))
FIX = os.path.join(HERE, "fix"); REMOTE = os.path.join(HERE, "remote.git")
for d in (FIX, REMOTE):
    shutil.rmtree(d, ignore_errors=True)
NF, NC = 2000, 5000
files = [f"src/mod{i%50:02d}/file{i:04d}.txt" for i in range(NF)]
content = {f: "\n".join(f"line {j} of {f}" for j in range(40)) + "\n" for f in files}
out = []
def blob(data):
    b = data.encode()
    out.append(b"data %d\n" % len(b)); out.append(b)
mark = 0
t = 1600000000
def commit(branch, msg, changes, frm=None, merge=None):
    global mark, t
    mark += 1; t += 60
    out.append(f"commit refs/heads/{branch}\nmark :{mark}\nauthor Dev <dev@x> {t} +0000\ncommitter Dev <dev@x> {t} +0000\n".encode())
    blob(msg + "\n")
    if frm: out.append(f"from :{frm}\n".encode())
    if merge: out.append(f"merge :{merge}\n".encode())
    for f in changes:
        out.append(f"M 100644 inline {f}\n".encode()); blob(content[f])
    out.append(b"\n")
    return mark
main = commit("main", "initial", files); feat = None
for i in range(1, NC):
    ch = random.sample(files, random.randint(1, 4))
    for f in ch:
        lines = content[f].split("\n"); k = random.randrange(40); lines[k] = f"edit {i} " + lines[k]; content[f] = "\n".join(lines)
    if i % 100 == 10:
        feat = main
    if feat is not None and 10 <= i % 100 < 20:
        feat = commit(f"feature/f{i//100}", f"feature work {i}", ch, frm=feat); continue
    if feat is not None and i % 100 == 20:
        main = commit("main", f"Merge feature {i//100}", ch, frm=main, merge=feat); feat = None; continue
    main = commit("main", f"commit {i}: change {' '.join(os.path.basename(c) for c in ch)}", ch, frm=main)
# a big file for diff measurement
big = "\n".join(f"big line {j} " + "x" * 60 for j in range(20000)) + "\n"
commit("main", "add big file", [], frm=main)
data = b"".join(out)
def g(*a, cwd=None, inp=None):
    subprocess.run(["git", *a], cwd=cwd, input=inp, check=True)
g("init", "-q", "-b", "main", FIX)
g("fast-import", "--quiet", cwd=FIX, inp=data)
g("checkout", "-q", "main", cwd=FIX)
with open(os.path.join(FIX, "big.txt"), "w", newline="\n") as fh: fh.write(big)
g("add", "big.txt", cwd=FIX)
g("-c", "user.name=Dev", "-c", "user.email=dev@x", "commit", "-q", "-m", "big file", cwd=FIX)
g("clone", "-q", "--bare", FIX, REMOTE)
g("remote", "add", "origin", REMOTE, cwd=FIX)
g("fetch", "-q", "origin", cwd=FIX)
g("branch", "-q", "-u", "origin/main", cwd=FIX)
g("config", "user.name", "Dev", cwd=FIX); g("config", "user.email", "dev@x", cwd=FIX)
print("ok")
