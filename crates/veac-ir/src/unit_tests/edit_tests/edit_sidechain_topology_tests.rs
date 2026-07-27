use super::*;

#[path = "edit_sidechain_topology_tests/lifecycle.rs"]
mod lifecycle;
#[path = "edit_sidechain_topology_tests/overwrite.rs"]
mod overwrite;

const KEY_TRACK: &str = "trk_video";
const RELATION: &str = "rel_sidechain_edit";
const RIGHT_ITEM: &str = "itm_sidechain_right";
const RIGHT_RELATION: &str = "rel_sidechain_right";
const TARGET: &str = "itm_sidechain_target";
const TARGET_TRACK: &str = "trk_sidechain_target";

fn sidechain_project(active_range: Option<TimeRange>) -> ProjectEnvelope {
    let mut project = sample_project();
    let sequence = &mut project.project.sequences[0];
    let mut track = sequence.tracks[0].clone();
    track.id = track_id(TARGET_TRACK);
    track.kind = TrackKind::Audio;
    track.order = 1;
    track.clips[0].id = item_id(TARGET);
    track.clips[0].visual = None;
    track.clips[0].effects.clear();
    sequence.tracks.push(track);
    project.project.relations.push(Relation {
        id: relation_id(RELATION),
        sequence_id: sequence.id.clone(),
        kind: RelationKind::Sidechain {
            key: RelationEndpoint::track(track_id(KEY_TRACK)),
            target: RelationEndpoint::item(item_id(TARGET)),
            parameters: SidechainRelationParameters {
                threshold_db: -18.0,
                ratio: 4.0,
                attack_ms: 10.0,
                release_ms: 100.0,
                active_range,
            },
        },
    });
    assert_canonical(&project);
    project
}

fn split(project: &ProjectEnvelope, fragments: Vec<RelationFragmentId>) -> EditOutcome {
    apply_edit_batch(
        project,
        &batch(
            "op_sidechain_split",
            project,
            vec![EditOperation::SplitClip {
                clip_id: item_id(TARGET),
                at: time(300),
                right_clip_id: item_id(RIGHT_ITEM),
                relation_fragments: fragments,
            }],
        ),
    )
}

fn fragment(relation: &str, right: &str) -> RelationFragmentId {
    RelationFragmentId {
        relation_id: relation_id(relation),
        right_relation_id: relation_id(right),
    }
}

fn relation_id(value: &str) -> RelationId {
    RelationId::new(value).unwrap()
}

fn item_id(value: &str) -> ItemId {
    ItemId::new(value).unwrap()
}

fn track_id(value: &str) -> TrackId {
    TrackId::new(value).unwrap()
}

fn assert_canonical(project: &ProjectEnvelope) {
    validate(project).unwrap();
    let json = canonical_json(project).unwrap();
    assert_eq!(decode_canonical_json(&json).unwrap(), project.clone());
}

fn sidechain_state<'a>(project: &'a ProjectEnvelope, id: &str) -> (&'a str, Option<TimeRange>) {
    let relation = project
        .project
        .relations
        .iter()
        .find(|relation| relation.id.as_str() == id)
        .unwrap();
    let RelationKind::Sidechain {
        target, parameters, ..
    } = &relation.kind
    else {
        panic!("expected sidechain relation")
    };
    (target.item_id().unwrap().as_str(), parameters.active_range)
}

fn assert_rejected(outcome: EditOutcome) {
    assert!(matches!(outcome, EditOutcome::Rejected { .. }));
}

#[test]
fn target_split_partitions_and_rebases_item_local_active_ranges() {
    let cases = [
        (
            Some(range(50, 100)),
            vec![],
            vec![(RELATION, TARGET, Some(range(50, 100)))],
        ),
        (
            Some(range(350, 100)),
            vec![fragment(RELATION, RIGHT_RELATION)],
            vec![(RIGHT_RELATION, RIGHT_ITEM, Some(range(50, 100)))],
        ),
        (
            Some(range(200, 200)),
            vec![fragment(RELATION, RIGHT_RELATION)],
            vec![
                (RELATION, TARGET, Some(range(200, 100))),
                (RIGHT_RELATION, RIGHT_ITEM, Some(range(0, 100))),
            ],
        ),
        (
            None,
            vec![fragment(RELATION, RIGHT_RELATION)],
            vec![(RELATION, TARGET, None), (RIGHT_RELATION, RIGHT_ITEM, None)],
        ),
    ];
    for (active, fragments, expected) in cases {
        let project = sidechain_project(active);
        let result = applied(split(&project, fragments));
        assert_eq!(result.project.relations.len(), expected.len());
        for (relation, target, active_range) in expected {
            assert_eq!(sidechain_state(&result, relation), (target, active_range));
        }
        assert_canonical(&result);
    }
}

#[test]
fn split_mapping_is_exact_conflict_free_and_atomic() {
    let crossing = sidechain_project(Some(range(200, 200)));
    for fragments in [
        vec![],
        vec![fragment("rel_unknown", RIGHT_RELATION)],
        vec![fragment(RELATION, RELATION)],
        vec![
            fragment(RELATION, RIGHT_RELATION),
            fragment(RELATION, "rel_sidechain_other"),
        ],
    ] {
        assert_rejected(split(&crossing, fragments));
    }
    let left_only = sidechain_project(Some(range(50, 100)));
    assert_rejected(split(&left_only, vec![fragment(RELATION, RIGHT_RELATION)]));
    let operations = vec![
        EditOperation::SplitClip {
            clip_id: item_id(TARGET),
            at: time(300),
            right_clip_id: item_id(RIGHT_ITEM),
            relation_fragments: vec![fragment(RELATION, RIGHT_RELATION)],
        },
        EditOperation::MoveClip {
            clip_id: item_id("itm_missing"),
            record_start: time(0),
        },
    ];
    assert_rejected(apply_edit_batch(
        &crossing,
        &batch("op_sidechain_rollback", &crossing, operations),
    ));
    assert_eq!(crossing, sidechain_project(Some(range(200, 200))));
}
