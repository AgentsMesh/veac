use super::*;

#[test]
fn removing_item_or_track_endpoints_removes_the_relation() {
    for item in [PRODUCER, CONSUMER] {
        let project = matte_project();
        let result = applied(apply_edit_batch(
            &project,
            &batch(
                "op_matte_remove_item",
                &project,
                vec![EditOperation::RemoveClip {
                    clip_id: item_id(item),
                }],
            ),
        ));
        assert!(result.project.relations.is_empty());
        assert_canonical(&result);
    }
    for track in ["trk_video", CONSUMER_TRACK] {
        let project = matte_project();
        let edit = StructureEdit::RemoveTrack {
            track_id: track_id(track),
        };
        let result = applied(apply_edit_batch(
            &project,
            &batch(
                "op_matte_remove_track",
                &project,
                vec![EditOperation::EditStructure { edit }],
            ),
        ));
        assert!(result.project.relations.is_empty());
        assert_canonical(&result);
    }
}

#[test]
fn locked_producer_rejects_consumer_rewrites_atomically() {
    let mut project = matte_project();
    project.project.sequences[0].tracks[0].state.locked = true;
    let before = project.clone();
    assert_rejected(split(
        &project,
        CONSUMER,
        vec![fragment(RELATION, RIGHT_RELATION)],
    ));
    assert_eq!(project, before);
}
