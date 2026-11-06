//! Settings > Synchronization > Updates: the read-only "Installed with" row
//! under the channel row, and the rows the machine policy sets (shown as
//! "Set by your administrator", not editable).

use super::*;
use crate::updates;

#[test]
fn installed_with_is_a_read_only_row_right_under_the_channel_and_locked_rows_say_who_set_them() {
    let _lock = updates::TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let dir = tempfile::tempdir().unwrap();
    let st = state(dir.path());

    // Nothing read yet: the row is there and says so.
    updates::set_test_facts(None);
    let fields = st.sync_fields();
    let channel = fields
        .iter()
        .position(|f| f.target == Target::Global("updates.channel"))
        .expect("the channel row");
    let row = &fields[channel + 1];
    assert_eq!(row.label, "Installed with");
    assert_eq!(row.control, Control::Info("...".into()));
    assert_eq!(row.target, Target::None, "nothing to edit");
    assert!(fields[channel].coming.is_none(), "not locked");

    // Read, and the administrator set the mode and channel.
    updates::set_test_facts(Some(updates::TestFacts {
        channel: gitpoulpi_config::release::InstallChannel::Choco,
        mode: Some(gitpoulpi_config::UpdateMode::Notify),
        release_channel: Some(gitpoulpi_config::UpdateChannel::Stable),
    }));
    let fields = st.sync_fields();
    let row = &fields[channel + 1];
    assert_eq!(row.control, Control::Info("Chocolatey".into()));
    let mode = fields
        .iter()
        .find(|f| f.target == Target::Global("updates.mode"))
        .unwrap();
    assert_eq!(mode.coming.as_deref(), Some("Set by your administrator"));
    assert!(!mode.selectable(), "a locked row cannot be selected");
    let channel_row = &fields[channel];
    assert_eq!(
        channel_row.coming.as_deref(),
        Some("Set by your administrator")
    );
    updates::set_test_facts(None);
}
