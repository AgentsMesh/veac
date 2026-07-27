use super::*;

fn group(id: &str, members: &[&str]) -> Relation {
    Relation {
        id: RelationId::new(id).unwrap(),
        sequence_id: SequenceId::new("seq_main").unwrap(),
        kind: RelationKind::Group {
            members: members
                .iter()
                .map(|id| RelationEndpoint::Item {
                    item_id: ItemId::new(*id).unwrap(),
                })
                .collect(),
        },
    }
}

fn structure(edit: StructureEdit) -> EditOperation {
    EditOperation::EditStructure { edit }
}

#[test]
fn inserts_group_as_a_canonical_relation() {
    let project = linked_project();
    let edit = batch(
        "op_insert_group_relation",
        &project,
        vec![structure(StructureEdit::InsertRelation {
            relation: group("rel_group", &["itm_video", "itm_audio"]),
        })],
    );
    let result = applied(apply_edit_batch(&project, &edit));

    assert!(result
        .project
        .relations
        .iter()
        .any(|entry| entry.id.as_str() == "rel_group"));
}

#[test]
fn sets_and_removes_group_through_relation_crud() {
    let mut project = linked_project();
    project
        .project
        .relations
        .push(group("rel_group", &["itm_video", "itm_audio"]));
    let set = batch(
        "op_set_group_relation",
        &project,
        vec![structure(StructureEdit::SetRelation {
            relation: group("rel_group", &["itm_audio", "itm_video"]),
        })],
    );
    let updated = applied(apply_edit_batch(&project, &set));
    let relation = updated
        .project
        .relations
        .iter()
        .find(|entry| entry.id.as_str() == "rel_group")
        .unwrap();
    let RelationKind::Group { members } = &relation.kind else {
        panic!("expected group relation")
    };
    assert_eq!(
        members[0],
        RelationEndpoint::Item {
            item_id: ItemId::new("itm_audio").unwrap(),
        }
    );

    let remove = batch(
        "op_remove_group_relation",
        &updated,
        vec![structure(StructureEdit::RemoveRelation {
            relation_id: RelationId::new("rel_group").unwrap(),
        })],
    );
    let removed = applied(apply_edit_batch(&updated, &remove));
    assert!(removed
        .project
        .relations
        .iter()
        .all(|entry| entry.id.as_str() != "rel_group"));
}

#[test]
fn locked_endpoint_rejects_group_relation_insert_atomically() {
    let mut project = linked_project();
    project.project.sequences[0].tracks[0].state.locked = true;
    let edit = batch(
        "op_locked_group_relation",
        &project,
        vec![structure(StructureEdit::InsertRelation {
            relation: group("rel_group", &["itm_video", "itm_audio"]),
        })],
    );

    assert_rejected(apply_edit_batch(&project, &edit), "EDIT_REJECTED");
}
