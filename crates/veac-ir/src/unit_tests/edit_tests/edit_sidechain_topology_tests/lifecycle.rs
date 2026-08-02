use super::*;

#[test]
fn removed_item_and_track_endpoints_drop_the_relation() {
    let project = sidechain_project(None);
    let result = applied(apply_edit_batch(
        &project,
        &batch(
            "op_sidechain_remove_item",
            &project,
            vec![EditOperation::RemoveClip {
                clip_id: item_id(TARGET),
            }],
        ),
    ));
    assert!(result.project.relations.is_empty());
    assert_canonical(&result);

    for track in [KEY_TRACK, TARGET_TRACK] {
        let project = sidechain_project(None);
        let edit = StructureEdit::RemoveTrack {
            track_id: track_id(track),
        };
        let result = applied(apply_edit_batch(
            &project,
            &batch(
                "op_sidechain_remove_track",
                &project,
                vec![EditOperation::EditStructure { edit }],
            ),
        ));
        assert!(result.project.relations.is_empty());
        assert_canonical(&result);
    }
}

#[test]
fn locked_key_track_rejects_target_split_atomically() {
    let mut project = sidechain_project(None);
    project.project.sequences[0].tracks[0].state.locked = true;
    let before = project.clone();
    assert_rejected(split(&project, vec![fragment(RELATION, RIGHT_RELATION)]));
    assert_eq!(project, before);
}
