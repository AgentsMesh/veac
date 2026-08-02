use super::*;

fn split(right_relation: &str, fragments: Vec<LinkedSplitFragment>) -> EditOperation {
    EditOperation::SplitLinked {
        link_relation_id: RelationId::new("rel_primary").unwrap(),
        offset: time(300),
        fragments,
        right_link_relation_id: RelationId::new(right_relation).unwrap(),
    }
}

fn fragment(source: &str, right: &str) -> LinkedSplitFragment {
    LinkedSplitFragment {
        source_clip_id: ItemId::new(source).unwrap(),
        right_clip_id: ItemId::new(right).unwrap(),
        relation_fragments: vec![],
    }
}

fn valid_fragments(prefix: &str) -> Vec<LinkedSplitFragment> {
    vec![
        fragment("itm_video", &format!("itm_{prefix}_video")),
        fragment("itm_audio", &format!("itm_{prefix}_audio")),
    ]
}

#[test]
fn linked_split_clones_the_canonical_av_link() {
    let project = linked_project();
    let edit = batch(
        "op_linked_split",
        &project,
        vec![split("rel_right", valid_fragments("right"))],
    );
    let result = applied(apply_edit_batch(&project, &edit));
    let right = result
        .project
        .relations
        .iter()
        .find(|relation| relation.id.as_str() == "rel_right")
        .unwrap();
    let RelationKind::AvLink { video, audio } = &right.kind else {
        panic!("expected AV-link relation")
    };
    assert_eq!(
        video,
        &RelationEndpoint::Item {
            item_id: ItemId::new("itm_right_video").unwrap(),
        }
    );
    assert_eq!(
        audio,
        &[RelationEndpoint::Item {
            item_id: ItemId::new("itm_right_audio").unwrap(),
        }]
    );
    for track in &result.project.sequences[0].tracks[..2] {
        assert_eq!(track.clips[0].record_range, range(0, 300));
        assert_eq!(track.clips[1].record_range, range(300, 300));
    }
}

#[test]
fn linked_split_rejects_incomplete_or_grouped_membership() {
    let project = linked_project();
    let missing = batch(
        "op_linked_split_missing",
        &project,
        vec![split(
            "rel_missing_right",
            vec![fragment("itm_video", "itm_only_right")],
        )],
    );
    assert_rejected(apply_edit_batch(&project, &missing), "EDIT_REJECTED");

    let mut grouped = project.clone();
    grouped.project.relations.push(Relation {
        id: RelationId::new("rel_cross_group").unwrap(),
        sequence_id: SequenceId::new("seq_main").unwrap(),
        kind: RelationKind::Group {
            members: vec![
                RelationEndpoint::Item {
                    item_id: ItemId::new("itm_video").unwrap(),
                },
                RelationEndpoint::Item {
                    item_id: ItemId::new("itm_caption").unwrap(),
                },
            ],
        },
    });
    let edit = batch(
        "op_linked_split_grouped",
        &grouped,
        vec![split("rel_grouped_right", valid_fragments("grouped"))],
    );
    assert_rejected(apply_edit_batch(&grouped, &edit), "EDIT_REJECTED");
}

#[test]
fn linked_split_rejects_a_locked_partner_atomically() {
    let mut project = linked_project();
    project.project.sequences[0].tracks[1].state.locked = true;
    let before = project.clone();
    let edit = batch(
        "op_linked_split_locked",
        &project,
        vec![split("rel_locked_right", valid_fragments("locked"))],
    );
    assert_rejected(apply_edit_batch(&project, &edit), "EDIT_REJECTED");
    assert_eq!(project, before);
}
