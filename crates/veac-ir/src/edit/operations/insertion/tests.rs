use super::*;
use crate::test_support::sample_project;

#[test]
fn ripple_members_rejects_an_unknown_track() {
    let envelope = sample_project();
    let sequence = &envelope.project.sequences[0];
    let missing = TrackId::new("trk_missing_anchor").expect("track id");
    assert!(ripple_members(
        &envelope.project,
        &sequence.id,
        &missing,
        crate::test_support::time(0),
    )
    .is_err());
}
