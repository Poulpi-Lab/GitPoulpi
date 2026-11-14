**English**

# Several remotes for one branch

Many repositories live on more than one host: a main one (`origin`), a mirror on another host, a backup. GitPoulpi lets a branch push to all of them with one click, and optionally pull from more than one, without changing what plain git does. Nothing here needs an account, and everything is stored in the repository's own `.git/config`, so plain git and other tools keep working.

## Push targets and pull sources

- The **upstream** is git's own notion (`branch.<name>.remote` and `merge`). It is the *primary* remote of the branch. GitPoulpi never changes how it works.
- A **push target** is an extra remote the branch also pushes to. The Push button sends the branch to the upstream and to every push target.
- A **pull source** is an extra remote a pull also takes commits from. It is off unless you turn it on (the **Pull too** checkbox): merging two remotes you did not choose is not a default anybody wants.

A target can use another branch name on that remote (`backup` gets `feat-x`).

## The Remotes dialog

Right-click a branch and pick **Remotes of <branch>...** (also in the Push dropdown, the palette as `Git: Remotes of Branch...`, and by clicking the `⇉3` badge on the branch's row in the Refs section). It lists every remote of the repository:

| Column | Meaning |
|---|---|
| Remote, Branch | The remote and the branch name there. **Rename on remote...** changes that name for the selected row. |
| Push | Tick it to make the remote a push target of this branch. |
| Pull too | Tick it to make a pull also integrate that remote. The upstream is always pulled. |
| State | Where the branch stands against it: in sync, ahead, behind, new, unrelated. |

A row marked "remote gone" names a remote the branch still mentions but the repository no longer has; **Remove** forgets it. The same dialog sets the upstream (the **Upstream** list) and **Unset upstream** clears it. **Apply** writes everything as one undoable change.

A repository can also give every new branch the same extra targets: **Settings > Git & encoding** (the repository-specific page), "Extra push targets for new branches" (a checkbox per remote). They are copied into a branch when it first gets an upstream; existing branches are not touched.

## Pushing

- **Push button** on a branch with extra targets: pushes to all of them. When every remote simply takes the new commits, it runs at once and the progress view shows each remote turning to a check mark. If something needs a decision (a remote has commits you do not), the plan dialog opens instead.
- **Push ▾** (the arrow next to the button; also the button itself when **Push button** is set to **ask** in Settings):
  - `Push to all (origin, gitlab, backup)`
  - `Push to <remote> only`, one per remote of the branch
  - `Push to selected remotes...`: always there, for any branch, even one with a single remote
  - `Remotes of <branch>...`
- The branch's right-click menu and the palette (`Git: Push to All Targets`, `Git: Push to One Remote...`, `Git: Push to Selected Remotes...`) offer the same actions with the same names.

**Push to selected remotes...** opens a list of every remote of the repository, ticked with the branch's upstream and targets. It is for this push only; tick **Remember for this branch** to store the choice as the branch's push targets.

### The plan

Before a grouped push GitPoulpi looks at every remote (a quick query, and a download from that remote only when it differs) so a problem on the third remote shows before the first push starts. The dialog lists each remote with where the branch stands and what it will do:

| Situation | Default action | Also offered |
|---|---|---|
| Remote has nothing of the branch, or lacks some commits | Push | Skip |
| Remote is ahead (you have nothing new for it) | Fast-forward from it | Skip |
| Remote differs because you rebased or amended what you pushed | Force push (with lease) | Merge, Rebase, Skip |
| Remote has commits of someone else | Merge from it (Rebase when **Pull button** is set to rebase) | Rebase, Force push (with lease), Skip |
| History shares nothing with yours | Skip | Force push (with lease) |
| The host refused the branch (protected) | Skip | none: forcing would be refused too |

Each row's action is a list you can change (Left/Right steps through it, Enter opens it, `s` skips the row). A merge shows whether it would conflict and in which files. A **force push** is always confirmed, names the remote and how many commits it replaces, always uses a lease (never a bare force), and cannot be undone: the result tells which commit was replaced and the command that puts it back. **Skip problems** sets every troubled row to Skip. **Run** (Ctrl+Enter) starts.

The merges, fast-forwards and rebases run first, in the order of the list, so your branch reaches its final state once; then the pushes go out one remote at a time, the upstream first. There is no atomic "all or nothing" across remotes: each one ends with its own result, and a failure on one (network, a hook) does not stop the others. **Retry failed** (or `Git: Retry Failed Pushes`) tries those again. Closing the progress view never stops the run; `Git: Show Grouped Push` brings it back.

### Conflicts and Continue

If a merge or rebase of the run stops on conflicts, the dialog closes and the ordinary merge banner and conflict editor take over. A strip on the Log reads `Grouped push paused (gitlab): resolve and conclude, then Continue.` with **Continue** and **Abort group** buttons.

- Resolve the conflicts and conclude the merge or rebase (in GitPoulpi or with plain git).
- **Continue** is greyed until that is done, and it is **never pressed for you**: nothing continues by itself. Press it (or run `Git: Continue Grouped Push`) and the run pushes to every remote again from your new tip. After a rebase, a remote that still holds the old commits comes back to the plan dialog (it now needs a force push) instead of being forced silently.
- **Abort group** (confirmed; `Git: Abort Grouped Push`) throws away the merge or rebase the run started and forgets the run. Pushes already done stay.

The strip survives a restart of GitPoulpi, and the CLI sees the same saved run.

## Pulling

- The **Pull** button pulls the upstream exactly as before, unless the branch has pull sources: then it pulls the upstream and every source, in order, each in the mode set by **Pull button** in Settings (merge, fast-forward only or rebase). A source that cannot be fast-forwarded is merged (or rebased) like the upstream; with fast-forward only, a source that needs more asks first.
- **Pull ▾** adds `Pull from all sources (origin + gitlab)` when there are sources, and `Pull from <remote> only` for each remote of the branch: a one-off pull of that remote's branch, whatever the checkboxes say.
- The palette has `Git: Pull from All Sources` and `Git: Pull from One Remote...`.
- A pull that stops on conflicts leaves the same strip, labelled as a grouped pull.

## Settings

All four are on **Settings > Git & encoding**.

| Setting | Default | Meaning |
|---|---|---|
| **Push button** | direct | direct pushes at once; ask opens the Push menu first. |
| **Pull button** | merge | The mode of the Pull button (merge, fast-forward only or rebase), and of the pull sources. |
| **Check every remote before a push** | on | Look at every remote before a grouped push (the CLI follows it; `--no-preflight` turns it off for one call). |
| **Extra push targets for new branches** (per repository) | none | The remotes a new branch also pushes to. |

## From the command line

```
gitpoulpi branch upstream add gitlab --pull      # a push target that is also a pull source
gitpoulpi branch upstream list                    # what the branch is set up for
gitpoulpi push --all-targets                      # upstream plus every target
gitpoulpi push --to gitlab --to backup:feat-x     # exactly these, this once
gitpoulpi pull --all-sources                      # upstream plus every pull source
gitpoulpi remote-group plan                       # what a grouped push would do, nothing changed
gitpoulpi remote-group continue                   # after resolving and concluding a stop
```

When a remote needs a decision the command answers with the plan (exit 3) and changes nothing; answer it with `--resolve gitlab=merge`. See [the CLI reference](cli-reference.md) for every flag. The MCP server has the same tools.

## With plain git

The targets are plain config keys, so a script or an alias can use them without GitPoulpi:

```
git config --get-all branch.feature.gitpoulpi-push-to     # one value per line: remote or remote:branch

# push the current branch to its upstream and to every target
git config alias.pushall '!b=$(git symbolic-ref --short HEAD); git push; for t in $(git config --get-all branch.$b.gitpoulpi-push-to); do r=${t%%:*}; d=${t#*:}; [ "$d" = "$t" ] && d=$b; git push "$r" "$b:$d"; done'
```

A value is `<remote>` (same branch name) or `<remote>:<branch>`, which is what the `d=${t#*:}` line sorts out. The plain alias does not look at the remotes first, does not offer a merge and cannot continue after a conflict: that is what the dialog is for.

## Good to know

- Two remotes that point at the same address are noticed ("gitlab and mirror push to the same URL"); the second is skipped by default.
- A host with a protected branch refuses a force push; the row says so and offers Skip only.
