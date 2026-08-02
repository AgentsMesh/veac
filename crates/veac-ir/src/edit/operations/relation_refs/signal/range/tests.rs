use super::*;
use crate::test_support::sample_project;

#[test]
fn resolves_fragment_windows_and_rejects_missing_references() {
    let mut envelope = sample_project();
    let sequence = &mut envelope.project.sequences[0];
    let sequence_id = sequence.id.clone();
    let left = sequence.tracks[0].clips[0].clone();
    let mut right = left.clone();
    right.id = ItemId::new("itm_signal_right").expect("item id");
    right.record_range.start = left.record_range.end().expect("left end");
    sequence.tracks[0].clips.push(right.clone());

    let windows = item_windows(&envelope.project, &sequence_id, &left.id, &right.id)
        .expect("fragment windows");
    assert_eq!(windows[0].duration, left.record_range.duration);
    assert_eq!(windows[1].start, left.record_range.duration);

    assert!(item_windows(
        &envelope.project,
        &SequenceId::new("seq_missing").expect("sequence id"),
        &left.id,
        &right.id,
    )
    .is_err());
    assert!(item_windows(
        &envelope.project,
        &sequence_id,
        &left.id,
        &ItemId::new("itm_missing").expect("item id"),
    )
    .is_err());
}
