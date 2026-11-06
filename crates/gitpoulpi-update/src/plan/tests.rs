use super::*;
use crate::channel::How;
use gitpoulpi_config::policy::{merge, Policy};
use gitpoulpi_config::release::Marker;
use gitpoulpi_config::UpdatesConfig;
use std::path::PathBuf;

fn resolved(channel: Channel) -> Resolved {
    Resolved {
        channel,
        how: How::Marker,
        dir: PathBuf::from("/opt/gp"),
        exe_name: "gitpoulpi-tui".into(),
        marker: None,
    }
}

fn installed(c: InstallChannel) -> Resolved {
    resolved(Channel::Install(c))
}

fn eff() -> EffectiveUpdates {
    merge(&Policy::default(), &UpdatesConfig::default())
}

fn eff_with(policy: Policy) -> EffectiveUpdates {
    merge(&policy, &UpdatesConfig::default())
}

fn ctx() -> PlanContext {
    PlanContext {
        to_version: "1.4.2".into(),
        previous_version: Some("1.4.1".into()),
        apt_source_present: true,
        aur_helper: Some("yay".into()),
        binstall: false,
    }
}

fn run_argv(p: &Procedure) -> String {
    match p {
        Procedure::Run { argv, .. } => display_command(argv),
        other => panic!("not a Run: {other:?}"),
    }
}

fn shown(p: &Procedure) -> Vec<String> {
    match p {
        Procedure::Show { commands } => commands.clone(),
        other => panic!("not a Show: {other:?}"),
    }
}

#[test]
fn archive_and_inno_swap_in_place() {
    assert_eq!(
        plan(&installed(InstallChannel::Archive), &eff(), &ctx()),
        Procedure::Swap {
            elevation: Elevation::None,
            registry: None
        }
    );
    assert_eq!(
        plan(&installed(InstallChannel::InnoUser), &eff(), &ctx()),
        Procedure::Swap {
            elevation: Elevation::None,
            registry: Some(Hive::CurrentUser)
        }
    );
}

#[test]
fn inno_machine_swaps_through_uac_and_updates_the_hklm_key() {
    assert_eq!(
        plan(&installed(InstallChannel::InnoMachine), &eff(), &ctx()),
        Procedure::Swap {
            elevation: Elevation::Uac,
            registry: Some(Hive::LocalMachine)
        }
    );
}

#[test]
fn choco_runs_upgrade_elevated() {
    let p = plan(&installed(InstallChannel::Choco), &eff(), &ctx());
    assert_eq!(run_argv(&p), "choco upgrade gitpoulpi -y");
    assert!(matches!(
        p,
        Procedure::Run {
            elevation: Elevation::Uac,
            ..
        }
    ));
}

#[test]
fn choco_internal_uses_the_marker_feed_and_no_params() {
    let mut r = installed(InstallChannel::ChocoInternal);
    r.marker = Some(Marker {
        feed: Some("https://nexus.example/choco/".into()),
        ..Marker::new("1.4.1", "x86_64-pc-windows-msvc")
    });
    let p = plan(&r, &eff(), &ctx());
    assert_eq!(
        run_argv(&p),
        "choco upgrade gitpoulpi -y --source=https://nexus.example/choco/"
    );
    assert!(!run_argv(&p).contains("--params"));
}

#[test]
fn a_policy_check_url_beats_the_marker_feed_for_choco_internal() {
    let mut r = installed(InstallChannel::ChocoInternal);
    r.marker = Some(Marker {
        feed: Some("https://old.example/".into()),
        ..Marker::new("1.4.1", "t")
    });
    let e = eff_with(Policy {
        check_url: Some("https://policy.example/".into()),
        ..Policy::default()
    });
    assert!(run_argv(&plan(&r, &e, &ctx())).ends_with("--source=https://policy.example/"));
}

#[test]
fn winget_runs_the_exact_silent_upgrade() {
    let p = plan(&installed(InstallChannel::Winget), &eff(), &ctx());
    assert_eq!(
        run_argv(&p),
        "winget upgrade --id GitPoulpi.GitPoulpi --exact --silent \
         --accept-source-agreements --accept-package-agreements"
    );
}

#[test]
fn scoop_and_homebrew_run_their_upgrade() {
    assert_eq!(
        run_argv(&plan(&installed(InstallChannel::Scoop), &eff(), &ctx())),
        "scoop update gitpoulpi"
    );
    assert_eq!(
        run_argv(&plan(&installed(InstallChannel::Homebrew), &eff(), &ctx())),
        "brew upgrade gitpoulpi"
    );
}

#[test]
fn apt_shows_the_command_and_never_runs_sudo() {
    let p = plan(&installed(InstallChannel::Apt), &eff(), &ctx());
    assert_eq!(
        shown(&p),
        ["sudo apt update && sudo apt install --only-upgrade gitpoulpi"]
    );
}

#[test]
fn apt_without_the_repository_points_at_the_deb() {
    let mut c = ctx();
    c.apt_source_present = false;
    assert!(matches!(
        plan(&installed(InstallChannel::Apt), &eff(), &c),
        Procedure::Announce {
            reason: AnnounceReason::AptWithoutRepository,
            ..
        }
    ));
}

#[test]
fn msi_only_announces_and_carries_the_administrators_message() {
    let e = eff_with(Policy {
        message: Some("Updates are deployed by IT.".into()),
        ..Policy::default()
    });
    assert_eq!(
        plan(&installed(InstallChannel::Msi), &e, &ctx()),
        Procedure::Announce {
            reason: AnnounceReason::Managed,
            commands: vec![],
            message: Some("Updates are deployed by IT.".into())
        }
    );
}

#[test]
fn cargo_shows_binstall_when_present_else_install_git_with_the_tag() {
    let r = resolved(Channel::Cargo);
    assert_eq!(
        shown(&plan(&r, &eff(), &ctx())),
        [format!(
            "cargo install --locked --git {REPO_URL} --tag releases/v1.4.2 gitpoulpi-cli"
        )]
    );
    let mut c = ctx();
    c.binstall = true;
    assert_eq!(
        shown(&plan(&r, &eff(), &c)),
        ["cargo binstall gitpoulpi-cli"]
    );
}

#[test]
fn aur_uses_the_first_helper_else_makepkg_and_never_pacman_sy() {
    let r = installed(InstallChannel::Aur);
    assert_eq!(shown(&plan(&r, &eff(), &ctx())), ["yay -S gitpoulpi-bin"]);
    let mut paru = ctx();
    paru.aur_helper = Some("paru".into());
    assert_eq!(shown(&plan(&r, &eff(), &paru)), ["paru -S gitpoulpi-bin"]);
    let mut none = ctx();
    none.aur_helper = None;
    let steps = shown(&plan(&r, &eff(), &none));
    assert_eq!(steps.len(), 2);
    assert!(steps[0].starts_with("git clone https://aur.archlinux.org/gitpoulpi-bin"));
    assert!(steps[1].contains("makepkg -si"));
    assert!(steps.iter().all(|s| !s.contains("pacman")));
}

#[test]
fn docker_announces_the_hub_pull_built_from_the_constant() {
    let p = plan(&installed(InstallChannel::Docker), &eff(), &ctx());
    let Procedure::Announce {
        reason, commands, ..
    } = p
    else {
        panic!("not an announce");
    };
    assert_eq!(reason, AnnounceReason::Container);
    assert_eq!(
        commands,
        [
            "docker pull poulpilab/gitpoulpi:1.4.2",
            "docker pull poulpilab/gitpoulpi:latest"
        ]
    );
    assert_eq!(commands[0], format!("docker pull {DOCKER_IMAGE}:1.4.2"));
}

#[test]
fn legacy_read_only_announces_and_dev_does_nothing() {
    assert!(matches!(
        plan(&resolved(Channel::LegacyReadOnly), &eff(), &ctx()),
        Procedure::Announce {
            reason: AnnounceReason::LegacyReadOnly,
            ..
        }
    ));
    assert_eq!(
        plan(&resolved(Channel::Dev), &eff(), &ctx()),
        Procedure::Nothing(NothingReason::NoInstallRecord)
    );
}

#[test]
fn mode_off_hides_every_channel() {
    let e = eff_with(Policy {
        mode: Some(UpdateMode::Off),
        ..Policy::default()
    });
    for c in InstallChannel::ALL {
        assert_eq!(
            plan(&installed(c), &e, &ctx()),
            Procedure::Nothing(NothingReason::ModeOff),
            "{c:?}"
        );
        assert_eq!(
            revert_plan(&installed(c), &e, &ctx()),
            Procedure::Nothing(NothingReason::ModeOff)
        );
    }
}

#[test]
fn no_self_update_turns_swaps_into_announcements_and_runs_into_shows() {
    let e = eff_with(Policy {
        allow_self_update: Some(false),
        message: Some("IT decides.".into()),
        ..Policy::default()
    });
    for c in [
        InstallChannel::Archive,
        InstallChannel::InnoUser,
        InstallChannel::InnoMachine,
    ] {
        assert_eq!(
            plan(&installed(c), &e, &ctx()),
            Procedure::Announce {
                reason: AnnounceReason::PolicyBlocksSelfUpdate,
                commands: vec![],
                message: Some("IT decides.".into())
            }
        );
    }
    assert_eq!(
        shown(&plan(&installed(InstallChannel::Choco), &e, &ctx())),
        ["choco upgrade gitpoulpi -y"]
    );
    assert_eq!(
        shown(&plan(&installed(InstallChannel::Scoop), &e, &ctx())),
        ["scoop update gitpoulpi"]
    );
    // What was already only shown stays shown.
    assert!(matches!(
        plan(&installed(InstallChannel::Apt), &e, &ctx()),
        Procedure::Show { .. }
    ));
}

#[test]
fn auto_stages_in_the_background_only_for_swap_channels() {
    let user_auto = UpdatesConfig {
        mode: UpdateMode::Auto,
        ..UpdatesConfig::default()
    };
    let auto = merge(&Policy::default(), &user_auto);
    for c in InstallChannel::ALL {
        let p = plan(&installed(c), &auto, &ctx());
        let swap = matches!(p, Procedure::Swap { .. });
        assert_eq!(stages_in_background(&p, &auto), swap, "{c:?}");
    }
    let notify = eff();
    let p = plan(&installed(InstallChannel::Archive), &notify, &ctx());
    assert!(!stages_in_background(&p, &notify), "notify never stages");
}

#[test]
fn revert_per_channel() {
    let c = ctx();
    let e = eff();
    assert!(matches!(
        revert_plan(&installed(InstallChannel::Archive), &e, &c),
        Procedure::Swap { .. }
    ));
    assert_eq!(
        shown(&revert_plan(&installed(InstallChannel::Choco), &e, &c)),
        ["choco install gitpoulpi --version 1.4.1 --allow-downgrade -y"]
    );
    let mut internal = installed(InstallChannel::ChocoInternal);
    internal.marker = Some(Marker {
        feed: Some("https://nexus.example/choco/".into()),
        ..Marker::new("1.4.2", "t")
    });
    assert_eq!(
        shown(&revert_plan(&internal, &e, &c)),
        ["choco install gitpoulpi --version 1.4.1 --allow-downgrade -y --source=https://nexus.example/choco/"]
    );
    assert_eq!(
        shown(&revert_plan(&installed(InstallChannel::Winget), &e, &c)),
        ["winget install --id GitPoulpi.GitPoulpi --version 1.4.1"]
    );
    assert_eq!(
        run_argv(&revert_plan(&installed(InstallChannel::Scoop), &e, &c)),
        "scoop reset gitpoulpi@1.4.1"
    );
    assert_eq!(
        shown(&revert_plan(&installed(InstallChannel::Apt), &e, &c)),
        ["sudo apt install gitpoulpi=1.4.1"]
    );
    assert_eq!(
        shown(&revert_plan(&installed(InstallChannel::Docker), &e, &c)),
        ["docker pull poulpilab/gitpoulpi:1.4.1"]
    );
    assert_eq!(
        shown(&revert_plan(&resolved(Channel::Cargo), &e, &c)),
        [format!(
            "cargo install --locked --git {REPO_URL} --tag releases/v1.4.1 gitpoulpi-cli"
        )]
    );
    assert!(matches!(
        revert_plan(&installed(InstallChannel::Msi), &e, &c),
        Procedure::Announce {
            reason: AnnounceReason::Managed,
            ..
        }
    ));
    assert!(matches!(
        revert_plan(&installed(InstallChannel::Aur), &e, &c),
        Procedure::Announce {
            reason: AnnounceReason::NoRevert,
            ..
        }
    ));
}

#[test]
fn a_manager_revert_without_a_known_previous_version_cannot_guess_one() {
    let mut c = ctx();
    c.previous_version = None;
    assert!(matches!(
        revert_plan(&installed(InstallChannel::Choco), &eff(), &c),
        Procedure::Announce {
            reason: AnnounceReason::NoRevert,
            ..
        }
    ));
    // A file swap needs no version: the updater looks for the `.old` files.
    assert!(matches!(
        revert_plan(&installed(InstallChannel::Archive), &eff(), &c),
        Procedure::Swap { .. }
    ));
}

#[test]
fn a_blocked_self_update_blocks_the_revert_swap_too() {
    let e = eff_with(Policy {
        allow_self_update: Some(false),
        ..Policy::default()
    });
    assert!(matches!(
        revert_plan(&installed(InstallChannel::InnoUser), &e, &ctx()),
        Procedure::Announce {
            reason: AnnounceReason::PolicyBlocksSelfUpdate,
            ..
        }
    ));
    assert_eq!(
        shown(&revert_plan(&installed(InstallChannel::Scoop), &e, &ctx())),
        ["scoop reset gitpoulpi@1.4.1"]
    );
}

#[test]
fn commands_with_spaces_are_quoted_for_copying() {
    assert_eq!(
        display_command(&["choco".into(), "--source=C:\\my feed".into()]),
        "choco \"--source=C:\\my feed\""
    );
    assert_eq!(display_command(&["a".into(), "b".into()]), "a b");
}
