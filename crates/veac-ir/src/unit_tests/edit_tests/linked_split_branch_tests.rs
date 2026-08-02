use super::*;

fn split(right_relation_id: &str) -> EditOperation {
    EditOperation::SplitLinked {
        link_relation_id: RelationId::new("rel_primary").unwrap(),
        offset: time(300),
        fragments: vec![
            LinkedSplitFragment {
                source_clip_id: ItemId::new("itm_video").unwrap(),
                right_clip_id: ItemId::new("itm_branch_video").unwrap(),
                relation_fragments: vec![],
            },
            LinkedSplitFragment {
                source_clip_id: ItemId::new("itm_audio").unwrap(),
                right_clip_id: ItemId::new("itm_branch_audio").unwrap(),
                relation_fragments: vec![],
            },
        ],
        right_link_relation_id: RelationId::new(right_relation_id).unwrap(),
    }
}

#[test]
fn linked_split_reports_the_inserted_relation() {
    let project = linked_project();
    let edit = batch(
        "op_linked_change",
        &project,
        vec![split("rel_branch_right")],
    );
    let EditOutcome::Applied {
        changed_objects, ..
    } = apply_edit_batch(&project, &edit)
    else {
        panic!("expected linked split to apply")
    };
    assert!(changed_objects.contains(&ChangedObjectId::Relation {
        id: RelationId::new("rel_branch_right").unwrap(),
    }));
}

#[test]
fn linked_split_rejects_a_duplicate_right_relation_id() {
    let project = linked_project();
    let edit = batch("op_linked_duplicate", &project, vec![split("rel_primary")]);

    assert_rejected(apply_edit_batch(&project, &edit), "EDIT_REJECTED");
}
