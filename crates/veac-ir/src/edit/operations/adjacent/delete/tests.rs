use crate::test_support::{sample_project, time};
use crate::*;

#[test]
fn ripple_delete_orders_same_track_group_members_before_shifting() {
    let mut project = sample_project();
    let sequence = &mut project.project.sequences[0];
    let track = &mut sequence.tracks[0];
    track.clips[0].effects.clear();
    track.clips[0].visual.as_mut().unwrap().opacity = Animatable::constant(1.0);
    let mut middle = track.clips[0].clone();
    middle.id = ItemId::new("itm_middle").unwrap();
    middle.record_range.start = time(600);
    let mut right = middle.clone();
    right.id = ItemId::new("itm_right").unwrap();
    right.record_range.start = time(1200);
    track.clips.extend([middle, right]);
    project.project.relations.push(Relation {
        id: RelationId::new("rel_pair").unwrap(),
        sequence_id: SequenceId::new("seq_main").unwrap(),
        kind: RelationKind::Group {
            members: vec![
                RelationEndpoint::Item {
                    item_id: ItemId::new("itm_video").unwrap(),
                },
                RelationEndpoint::Item {
                    item_id: ItemId::new("itm_middle").unwrap(),
                },
            ],
        },
    });
    let edit = EditBatch {
        operation_id: OperationId::new("op_delete_pair").unwrap(),
        base_revision: project.project.revision,
        atomic: true,
        preconditions: vec![],
        operations: vec![EditOperation::RippleDelete {
            clip_id: ItemId::new("itm_video").unwrap(),
        }],
    };
    let EditOutcome::Applied {
        project: result,
        changed_objects,
        ..
    } = apply_edit_batch(&project, &edit)
    else {
        panic!("delete should apply")
    };
    let sequence = &result.project.sequences[0];
    assert!(result.project.relations.is_empty());
    assert_eq!(sequence.tracks[0].clips.len(), 1);
    assert_eq!(sequence.tracks[0].clips[0].id.as_str(), "itm_right");
    assert_eq!(sequence.tracks[0].clips[0].record_range.start, time(0));
    assert!(changed_objects.iter().any(|change| matches!(
        change,
        ChangedObjectId::Relation { id } if id.as_str() == "rel_pair"
    )));
}
