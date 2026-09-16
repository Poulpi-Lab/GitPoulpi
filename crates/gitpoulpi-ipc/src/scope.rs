//! `ChangeScope` lives in `gitpoulpi-core` (the operation table in
//! `OperationKind::traits` names it); the wire type is re-exported here.

pub use gitpoulpi_core::domain::ChangeScope;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Event;

    #[test]
    fn default_is_everything_and_old_events_parse() {
        assert_eq!(ChangeScope::default(), ChangeScope::ALL);
        let old = r#"{"RepoChanged":{"repo":"x"}}"#;
        match serde_json::from_str::<Event>(old).unwrap() {
            Event::RepoChanged { scope, .. } => assert_eq!(scope, ChangeScope::ALL),
            other => panic!("{other:?}"),
        }
    }
}
