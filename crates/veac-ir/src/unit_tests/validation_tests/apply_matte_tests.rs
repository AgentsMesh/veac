use super::*;
use crate::test_support::range;
use crate::test_support::time;

#[test]
fn apply_accepts_a_typed_matte_consumer() {
    let mut project = project_with_apply("trk_scope");
    project.project.relations.push(matte(
        RelationEndpoint::item(ItemId::new("itm_video").unwrap()),
        RelationEndpoint::apply(ApplyId::new("apl_grade").unwrap()),
    ));
    assert!(validate(&project).is_ok());
    let graph = RelationGraph::project(&project.project);
    let edge = graph
        .matte_for_apply(
            &SequenceId::new("seq_main").unwrap(),
            &ApplyId::new("apl_grade").unwrap(),
        )
        .expect("typed apply matte");
    assert_eq!(edge.producer.clip.id.as_str(), "itm_video");
}

#[test]
fn rejects_missing_or_wrong_role_apply_endpoints() {
    let mut missing = project_with_apply("trk_scope");
    missing.project.relations.push(matte(
        RelationEndpoint::item(ItemId::new("itm_video").unwrap()),
        RelationEndpoint::apply(ApplyId::new("apl_missing").unwrap()),
    ));
    assert_code(&validation_codes(&missing), "RELATION_ENDPOINT_NOT_FOUND");

    let mut wrong = project_with_apply("trk_scope");
    wrong.project.relations.push(matte(
        RelationEndpoint::apply(ApplyId::new("apl_grade").unwrap()),
        RelationEndpoint::item(ItemId::new("itm_scope").unwrap()),
    ));
    assert_code(&validation_codes(&wrong), "RELATION_ENDPOINT_TYPE");
}

#[test]
fn apply_endpoint_is_rejected_by_every_non_consumer_role() {
    let apply = RelationEndpoint::apply(ApplyId::new("apl_grade").unwrap());
    let item = RelationEndpoint::item(ItemId::new("itm_scope").unwrap());
    let invalid = [
        RelationKind::Transition {
            from: apply.clone(),
            to: item.clone(),
            transition: Transition {
                kind: TransitionKind::Dissolve,
                duration: time(60),
                alignment: TransitionAlignment::Centered,
            },
        },
        RelationKind::Sidechain {
            key: apply.clone(),
            target: item.clone(),
            parameters: SidechainRelationParameters {
                threshold_db: -18.0,
                ratio: 4.0,
                attack_ms: 20.0,
                release_ms: 200.0,
                active_range: None,
            },
        },
        RelationKind::Group {
            members: vec![apply.clone(), item.clone()],
        },
        RelationKind::AvLink {
            video: apply,
            audio: vec![item],
        },
    ];
    for (index, kind) in invalid.into_iter().enumerate() {
        let mut project = project_with_apply("trk_scope");
        project.project.relations.push(Relation {
            id: RelationId::new(format!("rel_apply_role_{index}")).unwrap(),
            sequence_id: SequenceId::new("seq_main").unwrap(),
            kind,
        });
        assert_code(&validation_codes(&project), "RELATION_ENDPOINT_TYPE");
    }
}

#[test]
fn rejects_apply_matte_self_dependency_and_short_source() {
    let mut self_dependent = project_with_apply("trk_video");
    self_dependent.project.relations.push(matte(
        RelationEndpoint::item(ItemId::new("itm_video").unwrap()),
        RelationEndpoint::apply(ApplyId::new("apl_grade").unwrap()),
    ));
    assert_code(
        &validation_codes(&self_dependent),
        "APPLY_MATTE_SELF_DEPENDENCY",
    );

    let mut short = project_with_apply("trk_scope");
    short.project.sequences[0].tracks[0].clips[0].record_range = range(0, 300);
    short.project.relations.push(matte(
        RelationEndpoint::item(ItemId::new("itm_video").unwrap()),
        RelationEndpoint::apply(ApplyId::new("apl_grade").unwrap()),
    ));
    assert_code(&validation_codes(&short), "MATTE_SOURCE_RANGE");
}

pub(super) fn project_with_apply(target: &str) -> ProjectEnvelope {
    let mut project = sample_project();
    let mut track = project.project.sequences[0].tracks[0].clone();
    track.id = TrackId::new("trk_scope").unwrap();
    track.order = 2;
    track.clips[0].id = ItemId::new("itm_scope").unwrap();
    track.clips[0].visual = Some(crate::test_support::identity_layout_visual());
    track.clips[0].effects.clear();
    project.project.sequences[0].tracks.push(track);
    project.project.sequences[0].applies.push(grade(target));
    project
}

pub(super) fn grade(target: &str) -> Apply {
    Apply {
        id: ApplyId::new("apl_grade").unwrap(),
        enabled: true,
        record_range: range(0, 600),
        target: ApplyTarget::Layer {
            track_id: TrackId::new(target).unwrap(),
        },
        stages: vec![ApplyStage {
            id: ApplyStageId::new("aps_grade").unwrap(),
            enabled: true,
            active_range: None,
            operation: ApplyOperation::Color {
                pipeline: super::apply_target_tests::empty_pipeline(),
            },
        }],
        mix: ApplyMix::default(),
    }
}

pub(super) fn matte(producer: RelationEndpoint, consumer: RelationEndpoint) -> Relation {
    Relation {
        id: RelationId::new("rel_apply_matte").unwrap(),
        sequence_id: SequenceId::new("seq_main").unwrap(),
        kind: RelationKind::Matte {
            producer,
            consumer,
            parameters: MatteRelationParameters {
                mode: TrackMatteMode::Alpha,
                invert: false,
            },
        },
    }
}
