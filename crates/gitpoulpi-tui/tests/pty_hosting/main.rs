//! End-to-end checks of the pull-request and issue sections, the Create
//! Pull Request form and the Clone dialog's hosted sources -- the real
//! binary, the real daemon, real `git`, driven through a pty by
//! `xtask::pty`, against a **local mock of a GitLab instance** (a
//! `TcpListener` in this process answering the `/api/v4` routes the
//! sections use). No network beyond 127.0.0.1.
//!
//! The rules are `pty_ui.rs`'s: nothing is located by a hard-coded
//! coordinate, nothing sleeps, and every wait describes the whole state
//! it is waiting for.

#[path = "../common/mod.rs"]
mod common;

#[cfg(feature = "local-issues")]
mod board;
mod chip;
mod commit_tokens;
mod composer_ref;
#[cfg(feature = "conversation")]
mod conversation;
mod failure_card;
mod flow_mr;
mod gerrit;
mod gitea;
mod host_edit;
mod integrations;
mod issue_create;
mod issues;
#[cfg(feature = "local-issues")]
mod local_issues;
mod meta_edit;
mod mock;
mod pr_create;
mod providers;
mod prs;
mod repo_picker;
mod review;
#[cfg(feature = "ai")]
mod review_ai;
mod review_diff;
#[cfg(all(feature = "ai", unix))]
mod review_options;
mod support;
mod terminology;
