//! The words of the Radar, one function per kind of label, and the line a
//! repository says about itself.

use super::*;

pub(crate) fn act_label(act: Act) -> &'static str {
    match act {
        Act::OpenBrowser => msg::radar::ACT_OPEN_BROWSER.get(),
        Act::OpenRepo => msg::radar::ACT_OPEN_REPO.get(),
        Act::Checkout => msg::radar::ACT_CHECKOUT.get(),
        Act::Worktree => msg::radar::ACT_WORKTREE.get(),
        Act::Show => msg::radar::ACT_SHOW.get(),
        Act::StartWork => msg::radar::ACT_START_WORK.get(),
        Act::Pin => msg::radar::ACT_PIN.get(),
        Act::Snooze => msg::radar::ACT_SNOOZE.get(),
        Act::Unsnooze => msg::radar::ACT_UNSNOOZE.get(),
        Act::Link => msg::radar::ACT_LINK.get(),
        Act::Reconnect => msg::radar::ACT_RECONNECT.get(),
        Act::EditAssignees => msg::radar::ACT_EDIT_ASSIGNEES.get(),
        Act::EditReviewers => msg::radar::ACT_EDIT_REVIEWERS.get(),
        Act::EditLabels => msg::radar::ACT_EDIT_LABELS.get(),
        Act::SetMilestone => msg::radar::ACT_SET_MILESTONE.get(),
        Act::Review => msg::radar::ACT_REVIEW.get(),
        Act::ReviewApi => msg::radar::CHOICE_API.get(),
        Act::CloneReview => msg::radar::CHOICE_CLONE.get(),
        Act::TurnOn => msg::radar::ACT_TURN_ON.get(),
        Act::FitAllColumns => msg::log_draw::COLUMN_MENU_FIT_ALL.get(),
        // The menu row names the column (`column_menu_fit`); this is only the
        // fallback.
        Act::FitColumn(_) => msg::log_draw::COLUMN_MENU_FIT_ALL.get(),
    }
}

pub(super) fn view_label(view: View) -> &'static str {
    match view {
        View::All => msg::radar::VIEW_ALL.get(),
        View::Prs => msg::radar::VIEW_PRS.get(),
        View::Issues => msg::radar::VIEW_ISSUES.get(),
        View::Wips => msg::radar::VIEW_WIPS.get(),
        View::Snoozed => msg::radar::VIEW_SNOOZED.get(),
    }
}

pub(super) fn who_label(who: Who) -> &'static str {
    match who {
        Who::Everyone => msg::radar::WHO_EVERYONE.get(),
        Who::Mine => msg::radar::WHO_MINE.get(),
        Who::Assigned => msg::radar::WHO_ASSIGNED.get(),
        Who::ReviewRequested => msg::radar::WHO_REVIEW.get(),
        Who::Mentioned => msg::radar::WHO_MENTIONED.get(),
    }
}

/// What a request's checks and conflict state say, one phrase each; empty when
/// the host said nothing. The tooltip lists them, the detail line joins them.
pub(super) fn status_words(item: &Item) -> Vec<String> {
    use gitpoulpi_hosting::ChecksState;
    let mut out = Vec::new();
    match item.checks {
        Some(ChecksState::Passing) => out.push(msg::radar::CHECKS_PASSING.get().to_string()),
        Some(ChecksState::Failing) => out.push(msg::radar::CHECKS_FAILING.get().to_string()),
        Some(ChecksState::Pending) => out.push(msg::radar::CHECKS_PENDING.get().to_string()),
        None => {}
    }
    match item.has_conflicts {
        Some(true) => out.push(msg::radar::conflicts_with(item.target.clone())),
        Some(false) => out.push(msg::radar::NO_CONFLICTS.get().to_string()),
        None => {}
    }
    out
}

pub(super) fn snooze_label(how: SnoozeFor) -> &'static str {
    match how {
        SnoozeFor::Hour => msg::radar::SNOOZE_HOUR.get(),
        SnoozeFor::FourHours => msg::radar::SNOOZE_FOUR.get(),
        SnoozeFor::Day => msg::radar::SNOOZE_DAY.get(),
        SnoozeFor::UntilChanged => msg::radar::SNOOZE_CHANGED.get(),
    }
}

pub(super) fn snooze_key(how: SnoozeFor) -> &'static str {
    match how {
        SnoozeFor::Hour => "1h",
        SnoozeFor::FourHours => "4h",
        SnoozeFor::Day => "1d",
        SnoozeFor::UntilChanged => "changed",
    }
}

pub(super) fn snooze_from_key(key: &str) -> Option<SnoozeFor> {
    SnoozeFor::ALL.into_iter().find(|s| snooze_key(*s) == key)
}

pub(super) fn heading_title(heading: Heading) -> &'static str {
    match heading {
        Heading::Pinned => msg::radar::HEADING_PINNED.get(),
        Heading::Group(Group::NeedsReview) => msg::radar::HEADING_REVIEW.get(),
        Heading::Group(Group::Mine) => msg::radar::HEADING_MINE.get(),
        Heading::Group(Group::Assigned) => msg::radar::HEADING_ASSIGNED.get(),
        Heading::Group(Group::Mentioned) => msg::radar::HEADING_MENTIONED.get(),
        Heading::Group(Group::OtherPrs) => msg::radar::HEADING_OTHER.get(),
        Heading::Group(Group::IssueAssigned) => msg::radar::HEADING_ISSUES_ASSIGNED.get(),
        Heading::Group(Group::IssueCreated) => msg::radar::HEADING_ISSUES_CREATED.get(),
        Heading::Group(Group::Wip) => msg::radar::HEADING_WIP.get(),
        Heading::Snoozed => msg::radar::HEADING_SNOOZED.get(),
        Heading::Repos => msg::radar::HEADING_REPOS.get(),
    }
}

/// What a repository says about itself, if anything: a problem the user can
/// fix (Reconnect), the host's pause, the cache's age, or a provider its
/// remotes suggest (Link...). A linked repository that is fine says nothing;
/// neither does an unlinked one with no suggestion (most repositories).
pub(super) fn note_of(answer: &RepoAnswer, now: i64, folder_scope: bool) -> Option<Note> {
    use crate::hosting_task::{age_label, Problem};
    let snap = &answer.snapshot;
    let plain = |text: String, act| Note {
        path: answer.path.clone(),
        name: answer.name.clone(),
        text,
        act,
    };
    if let Some(why) = &answer.paused {
        return Some(plain(msg::radar::note_paused(why), None));
    }
    let problem = snap.problem.as_ref().or(snap.issues_problem.as_ref());
    if let Some(problem) = problem {
        let profile = snap
            .profile_id
            .clone()
            .or_else(|| snap.issues_target.as_ref().map(|t| t.profile.id.clone()));
        return Some(match problem {
            Problem::NoToken => plain(
                msg::hosting_ui::reconnect(msg::hosting_ui::WHY_NO_TOKEN.get()),
                Some((Act::Reconnect, profile)),
            ),
            Problem::Reconnect(_) => plain(
                msg::hosting_ui::reconnect(msg::hosting_ui::WHY_TOKEN_REJECTED.get()),
                Some((Act::Reconnect, profile)),
            ),
            Problem::Offline(_) => plain(
                msg::radar::note_offline(age_label(now, snap.fetched_at)),
                None,
            ),
            Problem::RateLimited(why) | Problem::Other(why) => plain(why.clone(), None),
        });
    }
    if !answer.linked() {
        let Some(suggestion) = snap.suggestions.first() else {
            // A folder scope lists repositories nobody chose: say that only
            // their WIP shows, and offer the link.
            return folder_scope.then(|| {
                plain(
                    msg::radar::NOTE_UNLINKED.get().to_string(),
                    Some((Act::Link, None)),
                )
            });
        };
        return Some(plain(
            msg::radar::note_link(suggestion.profile.label.clone()),
            Some((Act::Link, None)),
        ));
    }
    None
}
