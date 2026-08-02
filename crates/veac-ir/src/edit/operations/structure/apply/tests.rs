use super::*;
use crate::test_support::{sample_project, time};

fn test_apply() -> Apply {
    Apply {
        id: ApplyId::new("apl_structure_test").expect("apply id"),
        enabled: true,
        record_range: TimeRange::new(time(0), time(600)).expect("range"),
        target: ApplyTarget::ItemSet {
            item_ids: vec![ItemId::new("itm_video").expect("item id")],
        },
        stages: Vec::new(),
        mix: ApplyMix::default(),
    }
}

#[test]
fn rejects_unknown_sequence_and_apply_references() {
    let mut envelope = sample_project();
    let mut changed = ChangeSet::default();
    assert!(insert(
        &mut envelope.project,
        &SequenceId::new("seq_missing").expect("sequence id"),
        &test_apply(),
        &None,
        &None,
        &mut changed,
    )
    .is_err());
    assert!(locate(
        &envelope.project,
        &ApplyId::new("apl_missing").expect("apply id"),
    )
    .is_err());
}
