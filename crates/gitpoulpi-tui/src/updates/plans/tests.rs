use super::*;
use gitpoulpi_config::release::Marker;
use gitpoulpi_update::channel::How;
use std::path::PathBuf;

fn facts(channel: Channel, feed: Option<&str>) -> Facts {
    let mut marker = Marker::new("0.1.0", release::host_target());
    marker.feed = feed.map(str::to_string);
    Facts {
        resolved: Resolved {
            channel,
            how: How::Marker,
            dir: PathBuf::from("/opt/gp"),
            exe_name: "gitpoulpi-tui".into(),
            marker: Some(marker),
        },
        policy: Policy::default(),
        apt_source_present: true,
        aur_helper: None,
        binstall: false,
    }
}

fn user() -> UpdatesConfig {
    UpdatesConfig::default()
}

fn eff(policy: &Policy) -> EffectiveUpdates {
    policy::merge(policy, &user())
}

const API: &str = "https://api.github.com";

fn github(repo: &str) -> SourceSpec {
    SourceSpec::GitHubReleases {
        api_base: API.into(),
        repo: repo.into(),
        token: None,
    }
}

#[test]
fn every_channel_has_its_own_label() {
    let mut seen = std::collections::BTreeSet::new();
    for c in InstallChannel::ALL {
        assert!(seen.insert(channel_label(Channel::Install(c))), "{c:?}");
    }
    for c in [Channel::Cargo, Channel::LegacyReadOnly, Channel::Dev] {
        assert!(seen.insert(channel_label(c)), "{c:?}");
    }
}

#[test]
fn unknown_facts_plan_nothing() {
    let p = plans(None, &user(), "1.0.0", None);
    assert!(matches!(p.procedure, Procedure::Nothing(_)));
}

#[test]
fn an_archive_swaps_and_apt_shows_the_command() {
    let f = facts(Channel::Install(InstallChannel::Archive), None);
    let p = plans(Some(&f), &user(), "1.0.0", None);
    assert!(matches!(p.procedure, Procedure::Swap { .. }));
    let f = facts(Channel::Install(InstallChannel::Apt), None);
    let p = plans(Some(&f), &user(), "1.0.0", None);
    match p.procedure {
        Procedure::Show { commands } => assert!(commands[0].contains("apt install")),
        other => panic!("{other:?}"),
    }
}

#[test]
fn the_administrators_message_and_lock_come_through_the_policy() {
    let mut f = facts(Channel::Install(InstallChannel::Archive), None);
    f.policy.allow_self_update = Some(false);
    f.policy.message = Some("IT deploys updates".into());
    let p = plans(Some(&f), &user(), "1.0.0", None);
    match p.procedure {
        Procedure::Announce { message, .. } => {
            assert_eq!(message.as_deref(), Some("IT deploys updates"))
        }
        other => panic!("{other:?}"),
    }
}

#[test]
fn the_source_follows_the_channel_and_the_policy() {
    let policy = Policy::default();
    let e = eff(&policy);
    let repo = e.repo.clone();
    let pick = |f: &Facts, e: &EffectiveUpdates| choose_source(Some(f), e, API.into(), None);
    // The default: GitHub; a development build asks nobody.
    assert_eq!(
        pick(&facts(Channel::Install(InstallChannel::Archive), None), &e),
        github(&repo)
    );
    assert_eq!(pick(&facts(Channel::Dev, None), &e), SourceSpec::None);
    // Chocolatey: its community feed; internal: the marker's feed.
    assert_eq!(
        pick(&facts(Channel::Install(InstallChannel::Choco), None), &e),
        SourceSpec::NuGet {
            feed: CHOCOLATEY_COMMUNITY_FEED.into()
        }
    );
    assert_eq!(
        pick(
            &facts(
                Channel::Install(InstallChannel::ChocoInternal),
                Some("https://nexus.example/choco/")
            ),
            &e
        ),
        SourceSpec::NuGet {
            feed: "https://nexus.example/choco/".into()
        }
    );
    // The policy's `check` and `check_url` win over the channel.
    let manifest = Policy {
        check: Some(CheckKind::Manifest),
        check_url: Some("https://m.example/latest.json".into()),
        ..Policy::default()
    };
    assert_eq!(
        pick(
            &facts(Channel::Install(InstallChannel::Choco), None),
            &eff(&manifest)
        ),
        SourceSpec::Manifest {
            url: "https://m.example/latest.json".into()
        }
    );
    let none = Policy {
        check: Some(CheckKind::None),
        ..Policy::default()
    };
    assert_eq!(
        pick(
            &facts(Channel::Install(InstallChannel::Archive), None),
            &eff(&none)
        ),
        SourceSpec::None
    );
    // GitHub Enterprise: the policy's URL is the API root.
    let ghes = Policy {
        check: Some(CheckKind::Github),
        check_url: Some("https://ghe.example/api/v3".into()),
        ..Policy::default()
    };
    assert_eq!(
        pick(
            &facts(Channel::Install(InstallChannel::Archive), None),
            &eff(&ghes)
        ),
        SourceSpec::GitHubReleases {
            api_base: "https://ghe.example/api/v3".into(),
            repo,
            token: None
        }
    );
}

#[test]
fn mode_off_asks_nobody() {
    let policy = Policy {
        mode: Some(UpdateMode::Off),
        ..Policy::default()
    };
    let f = facts(Channel::Install(InstallChannel::Archive), None);
    assert_eq!(
        choose_source(Some(&f), &eff(&policy), API.into(), None),
        SourceSpec::None
    );
}
