mod support;

use support::*;
use veac_plan::canonical::*;
use veac_plan::ResolvedSidechain;

#[test]
fn sidechain_bus_parameters_range_and_relation_id_are_preserved() {
    let mut project = relation_project();
    let bus_id = BusId::new("bus_dialogue").unwrap();
    let active_range = Some(range(20, 400));
    add_sidechain(
        &mut project,
        "rel_bus",
        "seq_main",
        RelationEndpoint::bus(bus_id.clone()),
        "itm_video",
        SidechainRelationParameters {
            threshold_db: -21.0,
            ratio: 6.0,
            attack_ms: 12.0,
            release_ms: 345.0,
            active_range,
        },
    );

    let plan = resolved(&project);
    let actual = clip(&plan, "itm_video")
        .audio
        .as_ref()
        .unwrap()
        .sidechain
        .as_ref()
        .unwrap();
    assert_eq!(
        actual,
        &ResolvedSidechain {
            relation_id: RelationId::new("rel_bus").unwrap(),
            source: SidechainSource::Bus { bus_id },
            threshold_db: -21.0,
            ratio: 6.0,
            attack_ms: 12.0,
            release_ms: 345.0,
            active_range,
        }
    );
    assert!(clip(&plan, "itm_second")
        .audio
        .as_ref()
        .unwrap()
        .sidechain
        .is_none());
}

#[test]
fn different_targets_receive_only_their_relation_projections() {
    let mut project = relation_project();
    add_relation_set(&mut project);
    let plan = resolved(&project);
    let first = clip(&plan, "itm_video");
    let second = clip(&plan, "itm_second");

    let first_matte = first.visual.as_ref().unwrap().track_matte.as_ref().unwrap();
    assert_eq!(first_matte.relation_id.as_str(), "rel_matte_a");
    assert_eq!(first_matte.source_clip_id.as_str(), "itm_matte_a");
    let second_matte = second
        .visual
        .as_ref()
        .unwrap()
        .track_matte
        .as_ref()
        .unwrap();
    assert_eq!(second_matte.relation_id.as_str(), "rel_matte_b");
    assert_eq!(second_matte.source_clip_id.as_str(), "itm_matte_b");

    let first_sidechain = first.audio.as_ref().unwrap().sidechain.as_ref().unwrap();
    assert_eq!(first_sidechain.relation_id.as_str(), "rel_side_a");
    assert!(matches!(
        &first_sidechain.source,
        SidechainSource::Bus { bus_id } if bus_id.as_str() == "bus_dialogue"
    ));
    let second_sidechain = second.audio.as_ref().unwrap().sidechain.as_ref().unwrap();
    assert_eq!(second_sidechain.relation_id.as_str(), "rel_side_b");
    assert!(matches!(
        &second_sidechain.source,
        SidechainSource::Track { track_id } if track_id.as_str() == "trk_key"
    ));
}

#[test]
fn dangling_relations_are_rejected_before_projection() {
    let mut matte = relation_project();
    add_matte(
        &mut matte,
        "rel_dangling_matte",
        "seq_main",
        "itm_missing",
        "itm_video",
        matte_parameters(TrackMatteMode::Alpha, false),
    );
    assert_canonical_error(&matte, "RELATION_ENDPOINT_NOT_FOUND");

    let mut sidechain = relation_project();
    add_sidechain(
        &mut sidechain,
        "rel_dangling_sidechain",
        "seq_main",
        RelationEndpoint::bus(BusId::new("bus_missing").unwrap()),
        "itm_video",
        sidechain_parameters(-18.0, 4.0),
    );
    assert_canonical_error(&sidechain, "RELATION_ENDPOINT_NOT_FOUND");
}

#[test]
fn relation_vector_order_does_not_change_render_projection() {
    let mut forward = relation_project();
    add_relation_set(&mut forward);
    let mut reversed = forward.clone();
    reversed.project.relations.reverse();

    assert_eq!(
        relation_projection(&resolved(&forward)),
        relation_projection(&resolved(&reversed))
    );
}
