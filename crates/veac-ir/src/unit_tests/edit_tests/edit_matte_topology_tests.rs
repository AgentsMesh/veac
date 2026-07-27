use super::*;

#[path = "edit_matte_topology_tests/lifecycle.rs"]
mod lifecycle;
#[path = "edit_matte_topology_tests/overwrite.rs"]
mod overwrite;

const CONSUMER: &str = "itm_matte_consumer";
const CONSUMER_TRACK: &str = "trk_matte_consumer";
const PRODUCER: &str = "itm_video";
const RELATION: &str = "rel_matte_edit";
const RIGHT_ITEM: &str = "itm_matte_right";
const RIGHT_RELATION: &str = "rel_matte_right";

fn matte_project() -> ProjectEnvelope {
    let mut project = sample_project();
    let sequence = &mut project.project.sequences[0];
    let mut track = sequence.tracks[0].clone();
    track.id = track_id(CONSUMER_TRACK);
    track.order = 1;
    track.clips[0].id = item_id(CONSUMER);
    track.clips[0].effects.clear();
    track.clips[0].visual.as_mut().unwrap().opacity = Animatable::constant(1.0);
    sequence.tracks.push(track);
    project.project.relations.push(Relation {
        id: relation_id(RELATION),
        sequence_id: sequence.id.clone(),
        kind: RelationKind::Matte {
            producer: RelationEndpoint::item(item_id(PRODUCER)),
            consumer: RelationEndpoint::item(item_id(CONSUMER)),
            parameters: MatteRelationParameters {
                mode: TrackMatteMode::Alpha,
                invert: false,
            },
        },
    });
    assert_canonical(&project);
    project
}

fn split(project: &ProjectEnvelope, item: &str, fragments: Vec<RelationFragmentId>) -> EditOutcome {
    apply_edit_batch(
        project,
        &batch(
            "op_matte_split",
            project,
            vec![EditOperation::SplitClip {
                clip_id: item_id(item),
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

fn matte_endpoints<'a>(project: &'a ProjectEnvelope, id: &str) -> (&'a str, &'a str) {
    let relation = project
        .project
        .relations
        .iter()
        .find(|relation| relation.id.as_str() == id)
        .unwrap();
    let RelationKind::Matte {
        producer, consumer, ..
    } = &relation.kind
    else {
        panic!("expected matte relation")
    };
    (
        producer.item_id().unwrap().as_str(),
        consumer.item_id().unwrap().as_str(),
    )
}

fn assert_rejected(outcome: EditOutcome) {
    assert!(matches!(outcome, EditOutcome::Rejected { .. }));
}

#[test]
fn consumer_split_clones_the_explicit_relation_identity() {
    let project = matte_project();
    let result = applied(split(
        &project,
        CONSUMER,
        vec![fragment(RELATION, RIGHT_RELATION)],
    ));
    assert_eq!(result.project.relations.len(), 2);
    assert_eq!(matte_endpoints(&result, RELATION), (PRODUCER, CONSUMER));
    assert_eq!(
        matte_endpoints(&result, RIGHT_RELATION),
        (PRODUCER, RIGHT_ITEM)
    );
    assert_canonical(&result);
}

#[test]
fn split_relation_mapping_must_be_exact_new_and_unambiguous() {
    let project = matte_project();
    for fragments in [
        vec![],
        vec![fragment("rel_unknown", RIGHT_RELATION)],
        vec![fragment(RELATION, RELATION)],
        vec![
            fragment(RELATION, RIGHT_RELATION),
            fragment(RELATION, "rel_matte_other"),
        ],
    ] {
        assert_rejected(split(&project, CONSUMER, fragments));
    }
    assert_eq!(project, matte_project());
}

#[test]
fn producer_split_and_later_batch_failure_roll_back() {
    let project = matte_project();
    assert_rejected(split(
        &project,
        PRODUCER,
        vec![fragment(RELATION, RIGHT_RELATION)],
    ));
    let operations = vec![
        EditOperation::SplitClip {
            clip_id: item_id(CONSUMER),
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
        &project,
        &batch("op_matte_rollback", &project, operations),
    ));
    assert_eq!(project, matte_project());
}
