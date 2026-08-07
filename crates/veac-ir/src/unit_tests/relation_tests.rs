#[path = "relation_tests/cardinality_tests.rs"]
mod cardinality_tests;
#[path = "relation_tests/contract_tests.rs"]
mod contract_tests;
#[path = "relation_tests/graph_matte_sidechain_tests.rs"]
mod graph_matte_sidechain_tests;
#[path = "relation_tests/graph_tests.rs"]
mod graph_tests;
#[path = "relation_tests/membership_graph_tests.rs"]
mod membership_graph_tests;
#[path = "relation_tests/validation_tests.rs"]
mod validation_tests;

use crate::{
    test_support::{linked_project, range, time},
    *,
};

fn relation_project() -> ProjectEnvelope {
    let mut project = linked_project();
    let sequence = &mut project.project.sequences[0];
    let transition = Transition {
        kind: TransitionKind::Dissolve,
        duration: time(60),
        alignment: TransitionAlignment::Centered,
    };
    let mut next = sequence.tracks[0].clips[0].clone();
    next.id = item_id("itm_next");
    next.record_range = range(540, 600);
    next.effects.clear();
    next.visual.as_mut().unwrap().opacity = Animatable::constant(1.0);
    sequence.tracks[0].clips.push(next);

    sequence.tracks[2].clips[0].record_range = range(0, 600);
    let parameters = sidechain_parameters();

    project.project.relations.extend([
        relation(
            "rel_transition",
            RelationKind::Transition {
                from: item("itm_video"),
                to: item("itm_next"),
                transition,
            },
        ),
        relation(
            "rel_matte",
            RelationKind::Matte {
                producer: item("itm_caption"),
                consumer: item("itm_video"),
                parameters: MatteRelationParameters {
                    mode: TrackMatteMode::Alpha,
                    invert: false,
                },
            },
        ),
        relation(
            "rel_sidechain",
            RelationKind::Sidechain {
                key: RelationEndpoint::track(track_id("trk_audio")),
                target: item("itm_video"),
                parameters,
            },
        ),
        relation(
            "rel_group",
            RelationKind::Group {
                members: vec![item("itm_video"), item("itm_caption")],
            },
        ),
    ]);
    project
}

fn relation(id: &str, kind: RelationKind) -> Relation {
    Relation {
        id: RelationId::new(id).unwrap(),
        sequence_id: SequenceId::new("seq_main").unwrap(),
        kind,
    }
}

fn sidechain_parameters() -> SidechainRelationParameters {
    SidechainRelationParameters {
        threshold_db: -18.0,
        ratio: 4.0,
        attack_ms: 20.0,
        release_ms: 250.0,
        active_range: Some(range(0, 300)),
    }
}

fn relation_mut<'a>(project: &'a mut ProjectEnvelope, id: &str) -> &'a mut Relation {
    project
        .project
        .relations
        .iter_mut()
        .find(|relation| relation.id.as_str() == id)
        .unwrap()
}

fn relation_by_id<'a>(project: &'a ProjectEnvelope, id: &str) -> &'a Relation {
    project
        .project
        .relations
        .iter()
        .find(|relation| relation.id.as_str() == id)
        .unwrap()
}

fn item(value: &str) -> RelationEndpoint {
    RelationEndpoint::item(item_id(value))
}

fn item_id(value: &str) -> ItemId {
    ItemId::new(value).unwrap()
}

fn track_id(value: &str) -> TrackId {
    TrackId::new(value).unwrap()
}

fn codes(project: &ProjectEnvelope) -> Vec<String> {
    validate(project)
        .unwrap_err()
        .into_diagnostics()
        .into_iter()
        .map(|diagnostic| diagnostic.code)
        .collect()
}

fn assert_code(project: &ProjectEnvelope, expected: &str) {
    let actual = codes(project);
    assert!(actual.iter().any(|code| code == expected), "{actual:?}");
}
