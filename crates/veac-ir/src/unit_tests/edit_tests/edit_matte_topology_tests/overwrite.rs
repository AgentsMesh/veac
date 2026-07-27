use super::*;

const INSERTED: &str = "itm_matte_inserted";

fn overwrite(project: &ProjectEnvelope, span: TimeRange, split: bool) -> EditOutcome {
    let mut clip = project.project.sequences[0]
        .tracks
        .iter()
        .find(|track| track.id.as_str() == CONSUMER_TRACK)
        .unwrap()
        .clips[0]
        .clone();
    clip.id = item_id(INSERTED);
    clip.record_range = span;
    let split_fragments = split
        .then(|| OverwriteFragment {
            source_clip_id: item_id(CONSUMER),
            right_fragment_id: item_id(RIGHT_ITEM),
            relation_fragments: vec![fragment(RELATION, RIGHT_RELATION)],
        })
        .into_iter()
        .collect();
    apply_edit_batch(
        project,
        &batch(
            "op_matte_overwrite",
            project,
            vec![EditOperation::OverwriteClip {
                sequence_id: SequenceId::new("seq_main").unwrap(),
                track_id: track_id(CONSUMER_TRACK),
                clip: Box::new(clip),
                split_fragments,
            }],
        ),
    )
}

#[test]
fn overwrite_covers_removed_left_right_and_split_topologies() {
    let cases = [
        (range(0, 600), false, Vec::new()),
        (range(400, 200), false, vec![(RELATION, CONSUMER)]),
        (range(0, 200), false, vec![(RELATION, CONSUMER)]),
        (
            range(200, 200),
            true,
            vec![(RELATION, CONSUMER), (RIGHT_RELATION, RIGHT_ITEM)],
        ),
    ];
    for (span, split, expected) in cases {
        let project = matte_project();
        let result = applied(overwrite(&project, span, split));
        assert_eq!(result.project.relations.len(), expected.len());
        for (relation, consumer) in expected {
            assert_eq!(matte_endpoints(&result, relation), (PRODUCER, consumer));
        }
        assert_canonical(&result);
    }
}

#[test]
fn overwrite_relation_mapping_conflict_and_locked_producer_roll_back() {
    let project = matte_project();
    let mut locked = matte_project();
    locked.project.sequences[0].tracks[0].state.locked = true;
    let before = locked.clone();
    assert_rejected(overwrite(&locked, range(200, 200), true));
    assert_eq!(locked, before);

    let mut clip = project.project.sequences[0].tracks[2].clips[0].clone();
    clip.id = item_id(INSERTED);
    clip.record_range = range(200, 200);
    let fragment = OverwriteFragment {
        source_clip_id: item_id(CONSUMER),
        right_fragment_id: item_id(RIGHT_ITEM),
        relation_fragments: vec![fragment(RELATION, RELATION)],
    };
    let operation = apply_edit_batch(
        &project,
        &batch(
            "op_matte_overwrite_conflict",
            &project,
            vec![EditOperation::OverwriteClip {
                sequence_id: SequenceId::new("seq_main").unwrap(),
                track_id: track_id(CONSUMER_TRACK),
                clip: Box::new(clip),
                split_fragments: vec![fragment],
            }],
        ),
    );
    assert_rejected(operation);
}
