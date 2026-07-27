use crate::test_support::sample_project;

use super::*;

#[test]
fn adjacent_helpers_reject_missing_clips_and_invalid_timebases() {
    let mut project = sample_project().project;
    let missing = ItemId::new("itm_missing").unwrap();
    let error = unlocked_track(&mut project, &missing).unwrap_err();
    assert_eq!(error.code, "EDIT_REJECTED");
    assert!(error.message.contains("does not exist"));

    let invalid = RationalTime {
        value: 1,
        timescale: 0,
    };
    let error = negative(invalid, &missing).unwrap_err();
    assert_eq!(error.code, "EDIT_REJECTED");
    assert!(error.message.contains("timebase"));
}
