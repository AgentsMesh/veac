use super::*;

const INSERTED: &str = "itm_sidechain_inserted";

fn overwrite(project: &ProjectEnvelope, span: TimeRange, split: bool) -> EditOutcome {
    let mut clip = project.project.sequences[0]
        .tracks
        .iter()
        .find(|track| track.id.as_str() == TARGET_TRACK)
        .unwrap()
        .clips[0]
        .clone();
    clip.id = item_id(INSERTED);
    clip.record_range = span;
    let split_fragments = split
        .then(|| OverwriteFragment {
            source_clip_id: item_id(TARGET),
            right_fragment_id: item_id(RIGHT_ITEM),
            relation_fragments: vec![fragment(RELATION, RIGHT_RELATION)],
        })
        .into_iter()
        .collect();
    apply_edit_batch(
        project,
        &batch(
            "op_sidechain_overwrite",
            project,
            vec![EditOperation::OverwriteClip {
                sequence_id: SequenceId::new("seq_main").unwrap(),
                track_id: track_id(TARGET_TRACK),
                clip: Box::new(clip),
                split_fragments,
            }],
        ),
    )
}

#[test]
fn overwrite_rewrites_removed_left_right_and_split_active_ranges() {
    let cases = [
        (range(0, 600), Some(range(100, 100)), false, Vec::new()),
        (
            range(400, 200),
            Some(range(300, 200)),
            false,
            vec![(RELATION, TARGET, Some(range(300, 100)))],
        ),
        (
            range(0, 200),
            Some(range(100, 300)),
            false,
            vec![(RELATION, TARGET, Some(range(0, 200)))],
        ),
        (
            range(200, 200),
            Some(range(100, 400)),
            true,
            vec![
                (RELATION, TARGET, Some(range(100, 100))),
                (RIGHT_RELATION, RIGHT_ITEM, Some(range(0, 100))),
            ],
        ),
    ];
    for (span, active, split, expected) in cases {
        let project = sidechain_project(active);
        let result = applied(overwrite(&project, span, split));
        assert_eq!(result.project.relations.len(), expected.len());
        for (relation, target, active_range) in expected {
            assert_eq!(sidechain_state(&result, relation), (target, active_range));
        }
        assert_canonical(&result);
    }
}

#[test]
fn overwrite_split_mapping_conflict_and_locked_key_roll_back() {
    let project = sidechain_project(Some(range(100, 400)));
    let mut clip = project.project.sequences[0]
        .tracks
        .iter()
        .find(|track| track.id.as_str() == TARGET_TRACK)
        .unwrap()
        .clips[0]
        .clone();
    clip.id = item_id(INSERTED);
    clip.record_range = range(200, 200);
    let conflict = OverwriteFragment {
        source_clip_id: item_id(TARGET),
        right_fragment_id: item_id(RIGHT_ITEM),
        relation_fragments: vec![fragment(RELATION, RELATION)],
    };
    let outcome = apply_edit_batch(
        &project,
        &batch(
            "op_sidechain_overwrite_conflict",
            &project,
            vec![EditOperation::OverwriteClip {
                sequence_id: SequenceId::new("seq_main").unwrap(),
                track_id: track_id(TARGET_TRACK),
                clip: Box::new(clip),
                split_fragments: vec![conflict],
            }],
        ),
    );
    assert_rejected(outcome);

    let mut locked = sidechain_project(Some(range(100, 400)));
    locked.project.sequences[0].tracks[0].state.locked = true;
    let before = locked.clone();
    assert_rejected(overwrite(&locked, range(200, 200), true));
    assert_eq!(locked, before);
}
