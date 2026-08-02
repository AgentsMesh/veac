use super::*;
use crate::test_support::sample_project;

#[test]
fn resolves_item_duration_and_rejects_missing_references() {
    let envelope = sample_project();
    let sequence = &envelope.project.sequences[0];
    let clip = &sequence.tracks[0].clips[0];
    assert_eq!(
        item_duration(&envelope.project, &sequence.id, &clip.id).expect("duration"),
        clip.record_range.duration
    );
    assert!(item_duration(
        &envelope.project,
        &SequenceId::new("seq_missing").expect("sequence id"),
        &clip.id,
    )
    .is_err());
    assert!(item_duration(
        &envelope.project,
        &sequence.id,
        &ItemId::new("itm_missing").expect("item id"),
    )
    .is_err());
}
