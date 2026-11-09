//! The planner: from a workflow, the repository's facts and a request, the
//! list of steps an action would do. Pure; the daemon supplies the facts and
//! runs the steps.

use std::collections::{BTreeMap, BTreeSet};

use crate::domain::Oid;

use super::super::{
    BranchPattern, Classification, CompiledWorkflow, Fields, FinishMode, FinishTarget,
    MergeStrategy, ParentInfo, ParentSource, Token, Workflow,
};
use super::{FlowAction, FlowPlan, FlowRequest, FlowStep, HookInfo, MergeRequestSpec, SyncMethod};

/// What the planner needs to know about the repository.
#[derive(Debug, Clone, Default)]
pub struct FlowFacts {
    /// The checked-out local branch (`None`: detached).
    pub head: Option<String>,
    /// Local branches and their tips.
    pub branches: BTreeMap<String, Oid>,
    /// Remote-tracking branches by `remote/name`.
    pub remote_branches: BTreeMap<String, Oid>,
    /// Local branch -> the remote-tracking branch it tracks (`origin/x`).
    pub upstreams: BTreeMap<String, String>,
    pub tags: BTreeSet<String>,
    pub remotes: Vec<String>,
    /// Recorded parents (`branch.<b>.gitpoulpi-parent`, Git Town's).
    pub recorded: BTreeMap<String, (String, ParentSource)>,
    /// The working tree has changes.
    pub dirty: bool,
    /// `merge`, `rebase`, `cherry-pick` or `revert` while one runs.
    pub in_progress: Option<String>,
    /// The user name offered for a `{user}` token.
    pub user: Option<String>,
    /// `YYYY-MM-DD`, offered for a `{date}` token.
    pub today: String,
    /// The effective hooks are this user's own override, not the committed
    /// file's.
    pub hooks_personal: bool,
    /// The hash of the committed file's hooks (`hooks_hash`), when it has any.
    pub shared_hooks_hash: Option<String>,
}

/// A readable title for a topic branch: its slug as words, with the issue.
/// `fix-login` with issue `12` is `Fix login (#12)`; with an issue key,
/// `PROJ-12: Fix login`; without a slug, the branch name.
pub fn branch_title(fields: &Fields, branch: &str) -> String {
    let base = match fields.get("slug").filter(|s| !s.is_empty()) {
        Some(slug) => {
            let words = slug.replace(['-', '_', '/'], " ");
            let mut chars = words.chars();
            chars
                .next()
                .map(|c| c.to_uppercase().collect::<String>() + chars.as_str())
                .unwrap_or_default()
        }
        None => branch.to_string(),
    };
    if let Some(key) = fields.get("issue_key").filter(|k| !k.is_empty()) {
        format!("{key}: {base}")
    } else if let Some(issue) = fields.get("issue").filter(|i| !i.is_empty()) {
        format!("{base} (#{})", issue.trim_start_matches('#'))
    } else {
        base
    }
}

type Counts<'a> = &'a mut dyn FnMut(Oid, Oid) -> Option<(u32, u32)>;

/// Plans `request` against `workflow` and `facts`. `counts(tip, base)` is
/// `(ahead, behind)` of `tip` relative to `base`. A resume request is the
/// daemon's business (it holds the saved steps); this plans the rest.
pub fn plan_flow(
    workflow: &Workflow,
    facts: &FlowFacts,
    request: &FlowRequest,
    counts: Counts,
) -> FlowPlan {
    let mut p = Planner {
        wf: workflow,
        c: workflow.compile(),
        f: facts,
        r: request,
        counts,
        plan: FlowPlan {
            action: request.action,
            dirty: facts.dirty,
            ..FlowPlan::default()
        },
    };
    if workflow.is_empty() && request.action != FlowAction::PickUp {
        p.block("this repository has no branching workflow (see `flow config init`)");
        return p.finish_plan();
    }
    if let Some(op) = &facts.in_progress {
        p.block(format!("a {op} is in progress: finish or abort it first"));
    }
    match request.action {
        FlowAction::Start => p.start(),
        FlowAction::Finish => p.finish(),
        FlowAction::Sync => p.sync(),
        FlowAction::Promote => p.promote(),
        FlowAction::SetParent => p.set_parent(),
        FlowAction::PickUp => p.pick_up(),
    }
    p.finish_plan()
}

struct Planner<'a> {
    wf: &'a Workflow,
    c: CompiledWorkflow,
    f: &'a FlowFacts,
    r: &'a FlowRequest,
    counts: Counts<'a>,
    plan: FlowPlan,
}

impl Planner<'_> {
    fn block(&mut self, text: impl Into<String>) {
        self.plan.blockers.push(text.into());
    }

    fn warn(&mut self, text: impl Into<String>) {
        self.plan.warnings.push(text.into());
    }

    fn exists(&self, name: &str) -> bool {
        self.f.branches.contains_key(name)
    }

    /// The existing local branch a reference stands for: a base's name or
    /// alias, or a branch of exactly that name.
    fn concrete(&self, reference: &str) -> Option<String> {
        self.c
            .concrete_base(reference, &|n| self.f.branches.contains_key(n))
            .or_else(|| self.exists(reference).then(|| reference.to_string()))
    }

    fn parent_of(&self, branch: &str) -> Option<ParentInfo> {
        let class = self.c.classify(branch);
        let recorded = self.f.recorded.get(branch).map(|(n, s)| (n.as_str(), *s));
        self.c
            .resolve_parent(&class, recorded, &|n| self.f.branches.contains_key(n))
    }

    fn remote(&self) -> Option<String> {
        if let Some(r) = &self.r.remote {
            return Some(r.clone());
        }
        if self.f.remotes.iter().any(|r| r == "origin") {
            return Some("origin".to_string());
        }
        match self.f.remotes.as_slice() {
            [only] => Some(only.clone()),
            _ => None,
        }
    }

    /// How many commits the local `branch` lacks of its upstream.
    fn behind_upstream(&mut self, branch: &str) -> Option<(String, u32)> {
        let upstream = self.f.upstreams.get(branch)?.clone();
        let tip = *self.f.branches.get(branch)?;
        let up_tip = *self.f.remote_branches.get(&upstream)?;
        let (_, behind) = (self.counts)(tip, up_tip)?;
        Some((upstream, behind))
    }

    fn children_of(&self, branch: &str) -> Vec<String> {
        self.f
            .branches
            .keys()
            .filter(|b| b.as_str() != branch)
            .filter(|b| self.parent_of(b).is_some_and(|p| p.name == branch))
            .cloned()
            .collect()
    }

    fn goto(&self, steps: &mut Vec<FlowStep>, current: &mut Option<String>, branch: &str) {
        if current.as_deref() != Some(branch) {
            steps.push(FlowStep::Checkout {
                branch: branch.to_string(),
            });
            *current = Some(branch.to_string());
        }
    }

    fn hook_steps(&mut self, phase: &str, commands: &[String]) -> Vec<FlowStep> {
        let mut out = Vec::new();
        for command in commands {
            self.plan.hooks.push(HookInfo {
                phase: phase.to_string(),
                command: command.clone(),
                personal: self.f.hooks_personal,
            });
            if self.r.run_hooks {
                out.push(FlowStep::Hook {
                    phase: phase.to_string(),
                    command: command.clone(),
                    personal: self.f.hooks_personal,
                });
            }
        }
        self.plan.hooks_run = self.r.run_hooks && !self.plan.hooks.is_empty();
        out
    }

    fn finish_plan(mut self) -> FlowPlan {
        let mut irreversible = Vec::new();
        for step in &self.plan.steps {
            match step {
                FlowStep::Push { remote, branch, .. } => irreversible.push(format!(
                    "Pushing {branch} to {remote} is not taken back by Undo"
                )),
                FlowStep::DeleteRemoteBranch { remote, name } => irreversible.push(format!(
                    "Deleting {name} on {remote} is not taken back by Undo"
                )),
                FlowStep::OpenMergeRequest { .. } => irreversible.push(
                    "The merge request is opened on the host and Undo cannot close it".to_string(),
                ),
                FlowStep::Hook { command, .. } => irreversible.push(format!(
                    "The hook `{command}` can do anything: Undo only takes back git changes"
                )),
                _ => {}
            }
        }
        self.plan.undoable = irreversible.is_empty();
        self.plan.irreversible = irreversible;
        if self.f.dirty
            && self.plan.steps.iter().any(|s| {
                matches!(
                    s,
                    FlowStep::Merge { .. } | FlowStep::Rebase { .. } | FlowStep::Checkout { .. }
                )
            })
            && self.r.action != FlowAction::Start
        {
            self.plan.warnings.push(
                "The working tree has changes: they are stashed and restored when the \
                 auto-stash setting allows it, otherwise the action stops"
                    .to_string(),
            );
        }
        if !self.plan.hooks.is_empty() && !self.r.run_hooks {
            self.plan.warnings.push(
                "The workflow configures hooks that will not run (ask for them explicitly)"
                    .to_string(),
            );
        }
        self.plan
    }

    // ------------------------------------------------------------ start

    fn start(&mut self) {
        let Some(id) = self.r.topic.clone() else {
            self.block("start needs a branch type (`--type`)");
            return;
        };
        let Some(def) = self.c.topic(&id).cloned() else {
            let known: Vec<&str> = self.wf.topics.iter().map(|t| t.id.as_str()).collect();
            self.block(format!(
                "unknown branch type `{id}` (known: {})",
                known.join(", ")
            ));
            return;
        };
        let Some(pattern) = self.c.render_pattern(&id).cloned() else {
            self.block(format!("type `{id}` has no valid name pattern"));
            return;
        };
        let mut fields = self.r.fields.clone();
        fill_defaults(&pattern, &mut fields, self.f);
        let name = match pattern.render(&fields) {
            Ok(n) => n,
            Err(e) => {
                self.block(format!("cannot name the branch: {e}"));
                return;
            }
        };
        self.plan.branch = Some(name.clone());
        self.plan.title = format!("Start {} {name}", def.display_label());
        if self.exists(&name) {
            self.block(format!("a branch named {name} already exists"));
            return;
        }

        let parent_branch = def.parent.as_deref().and_then(|p| self.concrete(p));
        let from = match &self.r.from {
            Some(from) => {
                let known = self.exists(from)
                    || self.f.remote_branches.contains_key(from)
                    || self.f.tags.contains(from);
                if !known {
                    self.block(format!("the start point `{from}` is not a branch or a tag"));
                    return;
                }
                from.clone()
            }
            None => match &parent_branch {
                Some(p) => p.clone(),
                None => {
                    self.block(format!(
                        "type `{id}` has no existing parent branch to start from (pass `--from`)"
                    ));
                    return;
                }
            },
        };
        if self.exists(&from) {
            if let Some((upstream, behind)) = self.behind_upstream(&from).filter(|(_, b)| *b > 0) {
                self.warn(format!(
                    "{from} is {behind} commit{} behind {upstream}: fetch and update it first \
                     to start from the latest",
                    if behind == 1 { "" } else { "s" }
                ));
            }
        }
        let parent = if self.exists(&from) {
            Some(from.clone())
        } else {
            parent_branch
        };

        let mut steps = vec![
            FlowStep::CreateBranch {
                name: name.clone(),
                from: from.clone(),
            },
            FlowStep::Checkout {
                branch: name.clone(),
            },
        ];
        if parent.is_some() {
            steps.push(FlowStep::RecordParent {
                branch: name.clone(),
                parent,
            });
        }
        if self.r.publish {
            match self.remote() {
                Some(remote) => steps.push(FlowStep::Push {
                    remote,
                    branch: name.clone(),
                    set_upstream: true,
                }),
                None => self.block("there is no remote to publish to"),
            }
        }
        let hooks = self.wf.hooks.after_start.clone();
        steps.extend(self.hook_steps("after_start", &hooks));
        self.plan.steps = steps;
    }

    // ----------------------------------------------------------- finish

    fn finish(&mut self) {
        let Some(branch) = self.r.branch.clone().or_else(|| self.f.head.clone()) else {
            self.block("no branch to finish: HEAD is detached (pass a branch)");
            return;
        };
        self.plan.branch = Some(branch.clone());
        self.plan.title = format!("Finish {branch}");
        if !self.exists(&branch) {
            self.block(format!("there is no local branch {branch}"));
            return;
        }
        let (type_id, fields) = match self.c.classify(&branch) {
            Classification::Topic { type_id, fields } => (type_id, fields),
            Classification::Base { .. } => {
                self.block(format!(
                    "{branch} is a base branch: use `promote` or `sync` for it"
                ));
                return;
            }
            Classification::Unclassified => {
                self.block(format!(
                    "{branch} is not a topic branch of the workflow (no type matches its name)"
                ));
                return;
            }
        };
        let def = self.c.topic(&type_id).cloned().unwrap_or_default();
        let parent = self.parent_of(&branch);

        let mut targets: Vec<FinishTarget> = if def.finish.is_empty() {
            match &parent {
                Some(p) => vec![FinishTarget {
                    into: p.name.clone(),
                    merge: None,
                    tag: None,
                }],
                None => {
                    self.block(format!(
                        "type `{type_id}` has no finish target and {branch} has no parent"
                    ));
                    return;
                }
            }
        } else {
            def.finish.clone()
        };
        // A branch started from somewhere else finishes there first.
        if let Some(p) = parent
            .as_ref()
            .filter(|p| p.source == ParentSource::Recorded)
        {
            if let Some(first) = targets.first_mut() {
                first.into = p.name.clone();
            }
        }
        let mut resolved: Vec<(String, FinishTarget)> = Vec::new();
        for t in targets {
            match self.concrete(&t.into) {
                Some(name) if name == branch => {
                    self.block(format!("{branch} would be finished into itself"));
                }
                Some(name) => resolved.push((name, t)),
                None => self.block(format!("the finish target `{}` does not exist", t.into)),
            }
        }
        if !self.plan.blockers.is_empty() {
            return;
        }
        if let Some((upstream, behind)) = self.behind_upstream(&branch).filter(|(_, b)| *b > 0) {
            self.warn(format!(
                "{branch} is {behind} commit{} behind {upstream}: commits pushed there are not \
                 part of this finish (pull first)",
                if behind == 1 { "" } else { "s" }
            ));
        }
        let stacked = self.children_of(&branch);
        if !stacked.is_empty() {
            self.warn(format!(
                "{} branch{} stacked on {branch} ({}): re-parent them with `flow set-parent` \
                 or update them with `flow sync`",
                stacked.len(),
                if stacked.len() == 1 { " is" } else { "es are" },
                stacked.join(", ")
            ));
        }

        let mode = self
            .r
            .finish_mode
            .or(def.finish_mode)
            .unwrap_or(FinishMode::LocalMerge);
        let title = self
            .r
            .title
            .clone()
            .unwrap_or_else(|| branch_title(&fields, &branch));
        let mut steps: Vec<FlowStep> = Vec::new();
        let mut current = self.f.head.clone();

        let hooks = self.wf.hooks.before_finish.clone();
        let hook_steps = self.hook_steps("before_finish", &hooks);
        if !hook_steps.is_empty() {
            self.goto(&mut steps, &mut current, &branch);
            steps.extend(hook_steps);
        }

        match mode {
            FinishMode::Ask => {
                self.block(format!(
                    "type `{type_id}` asks how to finish: pass `--mode local-merge` or \
                     `--mode merge-request`"
                ));
            }
            FinishMode::MergeRequest => {
                let target = resolved[0].0.clone();
                match self.remote() {
                    Some(remote) => {
                        let set_upstream = !self.f.upstreams.contains_key(&branch);
                        steps.push(FlowStep::Push {
                            remote: remote.clone(),
                            branch: branch.clone(),
                            set_upstream,
                        });
                        steps.push(FlowStep::OpenMergeRequest {
                            source: branch.clone(),
                            target: target.clone(),
                            title: title.clone(),
                        });
                        self.plan.merge_request = Some(MergeRequestSpec {
                            source: branch.clone(),
                            target,
                            title,
                            remote,
                        });
                    }
                    None => self.block("there is no remote to open a merge request from"),
                }
                if self.r.delete == Some(true) {
                    self.warn(
                        "The branch is kept: a merge request needs it until it is merged"
                            .to_string(),
                    );
                }
            }
            FinishMode::LocalMerge => {
                self.local_merge(
                    &branch,
                    &def,
                    &fields,
                    &title,
                    &resolved,
                    parent,
                    &mut steps,
                    &mut current,
                );
            }
        }
        self.plan.steps = steps;
    }

    #[allow(clippy::too_many_arguments)]
    fn local_merge(
        &mut self,
        branch: &str,
        def: &crate::workflow::TopicType,
        fields: &Fields,
        title: &str,
        targets: &[(String, FinishTarget)],
        parent: Option<ParentInfo>,
        steps: &mut Vec<FlowStep>,
        current: &mut Option<String>,
    ) {
        for (target, spec) in targets {
            if !self.r.allow_behind {
                if let Some((upstream, behind)) =
                    self.behind_upstream(target).filter(|(_, b)| *b > 0)
                {
                    self.block(format!(
                        "{target} is {behind} commit{} behind {upstream}: update it first (or \
                         allow it)",
                        if behind == 1 { "" } else { "s" }
                    ));
                }
            }
            let strategy = spec
                .merge
                .or_else(|| self.c.base_named(target).and_then(|b| b.merge))
                .unwrap_or(MergeStrategy::Merge);
            match strategy {
                MergeStrategy::Rebase => {
                    steps.push(FlowStep::Rebase {
                        branch: branch.to_string(),
                        onto: target.clone(),
                    });
                    *current = Some(branch.to_string());
                    self.goto(steps, current, target);
                    steps.push(FlowStep::Merge {
                        branch: branch.to_string(),
                        into: target.clone(),
                        strategy: MergeStrategy::FastForward,
                        no_ff: false,
                        message: None,
                    });
                }
                MergeStrategy::Squash => {
                    self.goto(steps, current, target);
                    steps.push(FlowStep::Merge {
                        branch: branch.to_string(),
                        into: target.clone(),
                        strategy,
                        no_ff: false,
                        message: Some(title.to_string()),
                    });
                }
                MergeStrategy::Merge | MergeStrategy::FastForward => {
                    self.goto(steps, current, target);
                    steps.push(FlowStep::Merge {
                        branch: branch.to_string(),
                        into: target.clone(),
                        strategy,
                        no_ff: strategy == MergeStrategy::Merge,
                        message: None,
                    });
                }
            }
            if let Some(pattern) = &spec.tag {
                match BranchPattern::parse(pattern).and_then(|p| p.render(fields)) {
                    Ok(tag) if self.f.tags.contains(&tag) => {
                        self.block(format!("the tag {tag} already exists"));
                    }
                    Ok(tag) => steps.push(FlowStep::Tag {
                        message: format!("{} {tag}", def.display_label()),
                        name: tag,
                        at: target.clone(),
                    }),
                    Err(e) => self.block(format!("cannot name the tag `{pattern}`: {e}")),
                }
            }
        }
        let dest = parent
            .map(|p| p.name)
            .filter(|p| p != branch)
            .or_else(|| targets.last().map(|(t, _)| t.clone()))
            .unwrap_or_default();
        self.goto(steps, current, &dest);
        let delete = self.r.delete.unwrap_or(def.delete_after_finish);
        if delete {
            steps.push(FlowStep::DeleteBranch {
                name: branch.to_string(),
            });
            if self.r.delete_remote {
                let remote = self
                    .f
                    .upstreams
                    .get(branch)
                    .and_then(|u| u.split_once('/').map(|(r, _)| r.to_string()))
                    .or_else(|| {
                        self.remote().filter(|r| {
                            self.f
                                .remote_branches
                                .contains_key(&format!("{r}/{branch}"))
                        })
                    });
                match remote {
                    Some(remote) => steps.push(FlowStep::DeleteRemoteBranch {
                        remote,
                        name: branch.to_string(),
                    }),
                    None => self.warn(format!("{branch} has no remote branch to delete")),
                }
            }
        } else if self.r.delete_remote {
            self.warn("The remote branch is kept: deleting it needs the local branch deleted too");
        }
    }

    // ------------------------------------------------------------- sync

    fn sync(&mut self) {
        let Some(branch) = self.r.branch.clone().or_else(|| self.f.head.clone()) else {
            self.block("no branch to update: HEAD is detached (pass a branch)");
            return;
        };
        self.plan.branch = Some(branch.clone());
        self.plan.title = format!("Update {branch} from its parent");
        if !self.exists(&branch) {
            self.block(format!("there is no local branch {branch}"));
            return;
        }
        let Some(parent) = self.parent_of(&branch) else {
            self.block(format!(
                "{branch} has no parent: record one with `flow set-parent`"
            ));
            return;
        };
        if let Some((upstream, behind)) = self.behind_upstream(&parent.name).filter(|(_, b)| *b > 0)
        {
            self.warn(format!(
                "{} is {behind} commit{} behind {upstream}: sync updates from the local branch \
                 (fetch and update it first to get the latest)",
                parent.name,
                if behind == 1 { "" } else { "s" }
            ));
        }
        let method = self
            .r
            .method
            .unwrap_or_else(|| self.default_method(&branch));
        let mut steps = Vec::new();
        let mut current = self.f.head.clone();
        let mut queue: Vec<(String, String, bool)> = vec![(branch.clone(), parent.name, true)];
        let mut seen: BTreeSet<String> = BTreeSet::new();
        let mut changed_parent = false;
        while let Some((b, p, root)) = (!queue.is_empty()).then(|| queue.remove(0)) {
            if !seen.insert(b.clone()) {
                continue;
            }
            let (Some(&bt), Some(&pt)) = (self.f.branches.get(&b), self.f.branches.get(&p)) else {
                self.block(format!("{b} or its parent {p} does not exist"));
                continue;
            };
            let behind = (self.counts)(bt, pt).map(|(_, behind)| behind);
            let up_to_date = behind == Some(0) && (root || !changed_parent);
            if up_to_date {
                self.warn(format!("{b} is already up to date with {p}"));
            } else {
                match method {
                    SyncMethod::Merge => {
                        self.goto(&mut steps, &mut current, &b);
                        steps.push(FlowStep::Merge {
                            branch: p.clone(),
                            into: b.clone(),
                            strategy: MergeStrategy::Merge,
                            no_ff: false,
                            message: None,
                        });
                    }
                    SyncMethod::Rebase => {
                        steps.push(FlowStep::Rebase {
                            branch: b.clone(),
                            onto: p.clone(),
                        });
                        current = Some(b.clone());
                        if self.f.upstreams.contains_key(&b) {
                            self.warn(format!(
                                "Rebasing {b} rewrites commits it has on its remote: publishing \
                                 it afterwards needs a force push (with lease)"
                            ));
                        }
                    }
                }
                changed_parent = true;
            }
            if self.r.stack {
                for child in self.children_of(&b) {
                    queue.push((child, b.clone(), false));
                }
            }
        }
        if let Some(original) = self.f.head.clone() {
            self.goto(&mut steps, &mut current, &original);
        }
        // Only checkouts left: nothing to update.
        if steps.iter().all(|s| matches!(s, FlowStep::Checkout { .. })) {
            steps.clear();
        }
        self.plan.steps = steps;
    }

    fn default_method(&self, branch: &str) -> SyncMethod {
        let strategy = match self.c.classify(branch) {
            Classification::Topic { type_id, .. } => self
                .c
                .topic(&type_id)
                .and_then(|t| t.finish.first())
                .and_then(|f| f.merge),
            Classification::Base { name } => self.c.base_named(&name).and_then(|b| b.merge),
            Classification::Unclassified => None,
        };
        if strategy == Some(MergeStrategy::Rebase) {
            SyncMethod::Rebase
        } else {
            SyncMethod::Merge
        }
    }

    // ---------------------------------------------------------- promote

    fn promote(&mut self) {
        let Some(branch) = self.r.branch.clone() else {
            self.block("promote needs the environment branch to update (`flow promote <branch>`)");
            return;
        };
        self.plan.branch = Some(branch.clone());
        self.plan.title = format!("Promote into {branch}");
        let Some(env) = self.concrete(&branch) else {
            self.block(format!("there is no local branch {branch}"));
            return;
        };
        let Classification::Base { name } = self.c.classify(&env) else {
            self.block(format!("{env} is not a base branch of the workflow"));
            return;
        };
        let Some(parent) = self.parent_of(&env) else {
            self.block(format!("{name} has no parent branch to promote from"));
            return;
        };
        if !self.r.allow_behind {
            if let Some((upstream, behind)) = self.behind_upstream(&env).filter(|(_, b)| *b > 0) {
                self.block(format!(
                    "{env} is {behind} commit{} behind {upstream}: update it first (or allow it)",
                    if behind == 1 { "" } else { "s" }
                ));
            }
        }
        if let (Some(&et), Some(&pt)) =
            (self.f.branches.get(&env), self.f.branches.get(&parent.name))
        {
            if (self.counts)(pt, et).is_some_and(|(ahead, _)| ahead == 0) {
                self.warn(format!(
                    "{env} already contains everything of {}",
                    parent.name
                ));
                return;
            }
        }
        self.plan.title = format!("Promote {} into {env}", parent.name);
        let mut steps = Vec::new();
        let mut current = self.f.head.clone();
        self.goto(&mut steps, &mut current, &env);
        steps.push(FlowStep::Merge {
            branch: parent.name,
            into: env,
            strategy: MergeStrategy::Merge,
            no_ff: false,
            message: None,
        });
        if let Some(original) = self.f.head.clone() {
            self.goto(&mut steps, &mut current, &original);
        }
        self.plan.steps = steps;
    }

    // ---------------------------------------------------------- pick-up

    /// Takes over a branch a host or a bot made: the local branch tracks
    /// `remote/branch` (created when missing), the parent is recorded, the
    /// branch is checked out. The client fetched first.
    fn pick_up(&mut self) {
        let Some(branch) = self.r.branch.clone() else {
            self.block("pick-up needs the branch to take over");
            return;
        };
        self.plan.branch = Some(branch.clone());
        self.plan.title = format!("Pick up {branch}");
        let Some(remote) = self.remote() else {
            self.block("there is no remote to pick the branch up from");
            return;
        };
        let upstream = format!("{remote}/{branch}");
        let local = self.exists(&branch);
        let fetched = self.f.remote_branches.contains_key(&upstream);
        if !local && !fetched {
            self.block(format!(
                "{upstream} is not known here: fetch first, or the host has not made it"
            ));
            return;
        }
        let mut steps = Vec::new();
        if local {
            self.warn(format!(
                "a local branch {branch} exists already: it is checked out as it is"
            ));
        } else {
            steps.push(FlowStep::CreateBranch {
                name: branch.clone(),
                from: upstream.clone(),
            });
            steps.push(FlowStep::SetUpstream {
                branch: branch.clone(),
                upstream,
            });
        }
        steps.push(FlowStep::Checkout {
            branch: branch.clone(),
        });
        // The merge request's target, else what the workflow says for the
        // branch's type.
        let parent = self
            .r
            .parent
            .as_deref()
            .and_then(|p| self.concrete(p))
            .or_else(|| self.parent_of(&branch).map(|p| p.name))
            .filter(|p| *p != branch);
        match parent {
            Some(parent) => steps.push(FlowStep::RecordParent {
                branch: branch.clone(),
                parent: Some(parent),
            }),
            None => self.warn("no parent could be told for this branch; none is recorded"),
        }
        self.plan.steps = steps;
    }

    // ------------------------------------------------------- set-parent

    fn set_parent(&mut self) {
        let Some(branch) = self.r.branch.clone().or_else(|| self.f.head.clone()) else {
            self.block("no branch: HEAD is detached (pass a branch)");
            return;
        };
        self.plan.branch = Some(branch.clone());
        if !self.exists(&branch) {
            self.block(format!("there is no local branch {branch}"));
            return;
        }
        if self.r.clear {
            self.plan.title = format!("Forget the parent of {branch}");
            self.plan.steps = vec![FlowStep::RecordParent {
                branch,
                parent: None,
            }];
            return;
        }
        let Some(parent) = self.r.parent.clone() else {
            self.block("set-parent needs a parent branch (or `--clear`)");
            return;
        };
        self.plan.title = format!("Set the parent of {branch} to {parent}");
        let Some(parent) = self.concrete(&parent) else {
            self.block(format!("there is no local branch {parent}"));
            return;
        };
        if parent == branch {
            self.block("a branch cannot be its own parent");
            return;
        }
        // Walking up from the new parent must never come back to the branch.
        let mut at = parent.clone();
        for _ in 0..64 {
            match self.parent_of(&at) {
                Some(p) if p.name == branch => {
                    self.block(format!(
                        "{parent} descends from {branch}: that would be a cycle"
                    ));
                    return;
                }
                Some(p) => at = p.name,
                None => break,
            }
        }
        self.plan.steps = vec![FlowStep::RecordParent {
            branch,
            parent: Some(parent),
        }];
    }
}

/// `user` and `date` have natural values; the others the caller must give.
fn fill_defaults(pattern: &BranchPattern, fields: &mut Fields, facts: &FlowFacts) {
    for token in pattern.tokens() {
        let missing = fields.get(token.name()).is_none_or(|v| v.trim().is_empty());
        if !missing {
            continue;
        }
        match token {
            Token::User => {
                if let Some(user) = &facts.user {
                    fields.insert("user".into(), user.clone());
                }
            }
            Token::Date if !facts.today.is_empty() => {
                fields.insert("date".into(), facts.today.clone());
            }
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests;
