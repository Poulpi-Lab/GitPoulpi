use super::*;
use gitpoulpi_hosting::IntegrationKind;

mod ai_generate;
mod close;
mod form;
mod i18n;
mod mouse;
mod seed;
mod tabs;

fn remote(name: &str, url: &str) -> Remote {
    Remote {
        name: name.into(),
        fetch_url: url.into(),
        push_url: None,
    }
}

fn integration(id: &str, kind: IntegrationKind, url: &str) -> Integration {
    Integration::new(id, kind, url)
}

use crate::hosting_task::fake::{project, FakeFactory};
use crate::test_support::{local_branch_at, oid, remote_branch_at};

fn all_kinds() -> Vec<Integration> {
    vec![
        integration("hub", IntegrationKind::Github, "https://github.com"),
        integration(
            "corp",
            IntegrationKind::GitlabSelfManaged,
            "https://gitlab.example.com",
        ),
        integration("bb", IntegrationKind::Bitbucket, "https://bitbucket.org"),
        integration(
            "jira",
            IntegrationKind::JiraCloud,
            "https://x.atlassian.net",
        ),
    ]
}

fn seed_refs() -> Vec<Ref> {
    let mut head = local_branch_at("feature", oid(1));
    head.is_head = true;
    head.upstream = Some("origin/feature".into());
    vec![
        head,
        remote_branch_at("origin", "feature", oid(1)),
        remote_branch_at("origin", "main", oid(2)),
        remote_branch_at("origin", "develop", oid(3)),
    ]
}

fn remotes() -> Vec<Remote> {
    vec![remote("origin", "git@gitlab.example.com:g/p.git")]
}

fn dialog() -> PullRequestDialog {
    let seed = build_seed("feature", &seed_refs(), &remotes(), &all_kinds()).unwrap();
    PullRequestDialog::with_factory(
        seed,
        "Add the thing",
        std::sync::Arc::new(FakeFactory::new()),
    )
}

fn key(code: KeyCode) -> KeyEvent {
    KeyEvent::new(code, KeyModifiers::NONE)
}

fn paint(d: &mut PullRequestDialog, w: u16, h: u16) -> (String, Rect) {
    let mut buf = Buffer::empty(Rect::new(0, 0, w, h));
    let rect = pull_request_rect(buf.area);
    d.render(rect, &mut buf);
    (crate::dialog_kit::buffer_text(&buf), rect)
}
