use crate::edit::ChangeSet;
use crate::test_support::sample_project;

use super::*;

#[test]
fn curve_slice_rejects_an_overflowed_range_before_mutation() {
    let mut clip = sample_project().project.sequences[0].tracks[0].clips[0].clone();
    let original = clip.clone();
    let start = RationalTime::new(MAX_SAFE_INTEGER as i64, 600).unwrap();
    let duration = RationalTime::new(1, 600).unwrap();
    let owner = clip.id.clone();
    let error = crop_clip(
        &mut clip,
        start,
        duration,
        &owner,
        false,
        &mut ChangeSet::new(),
    )
    .unwrap_err();
    assert_eq!(error.code, "EDIT_REJECTED");
    assert!(error.message.contains("range is invalid"));
    assert_eq!(clip, original);
}
