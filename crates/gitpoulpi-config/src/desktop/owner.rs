//! Whose artifact is this? The pure rule every desktop writer (shortcuts,
//! aliases, folder-menu entries) applies before it rewrites or removes
//! anything: only what is [`Owner::Mine`] (or [`Owner::Adopt`]ed) is touched.
//!
//! Every artifact carries an owner tag ([`super::OWNER_TAG`]). Artifacts that
//! predate it carry only the legacy marker ([`super::MARKER`]) and belong to
//! the primary installation, if there is one.

use super::InstallCtx;

/// What was read off an artifact.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Facts<'a> {
    /// The value of the owner tag, when present.
    pub tag: Option<&'a str>,
    /// The legacy marker `X-GitPoulpi=1` is present.
    pub legacy_marker: bool,
    /// Neither tag nor marker, but it starts the running exe (a pin the user
    /// made by hand).
    pub names_my_exe: bool,
}

/// The verdict.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Owner {
    /// Tagged with the running installation's id: free to rewrite and remove.
    Mine,
    /// Not tagged, but the running installation may take it over: a legacy
    /// artifact while it is the primary, or a user's pin naming its exe (then
    /// only the `--launcher` flag is appended).
    Adopt,
    /// Another installation that still exists owns it: never touched.
    Other(String),
    /// Its installation is gone (`Some(id)`), or the tag is blank (`None`).
    /// Shown, and removable by the user.
    Orphan(Option<String>),
    /// Legacy marker, no primary: shown, never rewritten automatically.
    Legacy,
    /// Not ours at all.
    Foreign,
}

/// Decides who owns an artifact described by `facts`.
pub fn decide(facts: &Facts, ctx: &InstallCtx) -> Owner {
    let alive = |id: &str| ctx.known.iter().any(|k| k.id == id && k.exists);
    if let Some(tag) = facts.tag {
        let tag = tag.trim();
        return if tag.is_empty() {
            Owner::Orphan(None)
        } else if tag == ctx.id {
            Owner::Mine
        } else if alive(tag) {
            Owner::Other(tag.to_string())
        } else {
            Owner::Orphan(Some(tag.to_string()))
        };
    }
    if facts.legacy_marker {
        return match ctx.primary.as_deref() {
            None => Owner::Legacy,
            Some(p) if p == ctx.id => Owner::Adopt,
            Some(p) if alive(p) => Owner::Other(p.to_string()),
            Some(p) => Owner::Orphan(Some(p.to_string())),
        };
    }
    if facts.names_my_exe {
        Owner::Adopt
    } else {
        Owner::Foreign
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::desktop::KnownInstall;

    fn ctx(primary: Option<&str>) -> InstallCtx {
        InstallCtx {
            id: "aaaa".into(),
            short: "iaaaa".into(),
            name: "GitPoulpi (dev)".into(),
            primary: primary.map(str::to_string),
            known: vec![
                KnownInstall {
                    id: "aaaa".into(),
                    exists: true,
                },
                KnownInstall {
                    id: "bbbb".into(),
                    exists: true,
                },
                KnownInstall {
                    id: "cccc".into(),
                    exists: false,
                },
            ],
        }
    }

    fn tagged(tag: &str) -> Facts<'_> {
        Facts {
            tag: Some(tag),
            ..Facts::default()
        }
    }

    #[test]
    fn a_tagged_artifact_is_mine_only_with_my_id() {
        let c = ctx(None);
        assert_eq!(decide(&tagged("aaaa"), &c), Owner::Mine);
        assert_eq!(decide(&tagged("bbbb"), &c), Owner::Other("bbbb".into()));
        assert_ne!(decide(&tagged("bbbb"), &c), Owner::Mine);
        // The tag wins over the legacy marker.
        let both = Facts {
            tag: Some("bbbb"),
            legacy_marker: true,
            names_my_exe: true,
        };
        assert_eq!(decide(&both, &c), Owner::Other("bbbb".into()));
    }

    #[test]
    fn an_untagged_legacy_artifact_is_legacy_without_a_primary() {
        let legacy = Facts {
            legacy_marker: true,
            ..Facts::default()
        };
        assert_eq!(decide(&legacy, &ctx(None)), Owner::Legacy);
        // Another primary owns it.
        assert_eq!(
            decide(&legacy, &ctx(Some("bbbb"))),
            Owner::Other("bbbb".into())
        );
    }

    #[test]
    fn the_primary_adopts_untagged_legacy_artifacts() {
        let legacy = Facts {
            legacy_marker: true,
            ..Facts::default()
        };
        assert_eq!(decide(&legacy, &ctx(Some("aaaa"))), Owner::Adopt);
        // A pin the user made by hand names the exe: adopted by anyone running it.
        let pin = Facts {
            names_my_exe: true,
            ..Facts::default()
        };
        assert_eq!(decide(&pin, &ctx(None)), Owner::Adopt);
    }

    #[test]
    fn an_artifact_of_a_vanished_installation_is_an_orphan() {
        let c = ctx(None);
        // Known, but its exe is gone.
        assert_eq!(
            decide(&tagged("cccc"), &c),
            Owner::Orphan(Some("cccc".into()))
        );
        // Not in the config at all.
        assert_eq!(
            decide(&tagged("zzzz"), &c),
            Owner::Orphan(Some("zzzz".into()))
        );
        assert_eq!(decide(&tagged(" "), &c), Owner::Orphan(None));
        // A legacy artifact whose primary vanished.
        let legacy = Facts {
            legacy_marker: true,
            ..Facts::default()
        };
        assert_eq!(
            decide(&legacy, &ctx(Some("cccc"))),
            Owner::Orphan(Some("cccc".into()))
        );
    }

    #[test]
    fn no_marker_no_tag_is_foreign() {
        assert_eq!(
            decide(&Facts::default(), &ctx(Some("aaaa"))),
            Owner::Foreign
        );
        assert_eq!(decide(&Facts::default(), &ctx(None)), Owner::Foreign);
    }
}
