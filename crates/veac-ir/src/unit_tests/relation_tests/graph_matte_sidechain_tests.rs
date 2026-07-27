use super::*;

#[test]
fn matte_edges_are_stable_traceable_and_lookupable() {
    let mut project = relation_project();
    project.project.sequences[0].tracks[2].clips[0]
        .record_range
        .duration = time(1200);
    project.project.relations.push(relation(
        "rel_matte_next",
        RelationKind::Matte {
            producer: item("itm_caption"),
            consumer: item("itm_next"),
            parameters: MatteRelationParameters {
                mode: TrackMatteMode::Luma,
                invert: true,
            },
        },
    ));
    project.project.relations.reverse();
    validate(&project).unwrap();

    let sequence_id = SequenceId::new("seq_main").unwrap();
    let graph = RelationGraph::project(&project.project);
    let edges = graph.mattes(&sequence_id);
    assert_eq!(matte_ids(&edges), ["rel_matte", "rel_matte_next"]);
    assert_eq!(edges[0].sequence_id, &sequence_id);
    assert_eq!(edges[0].producer.clip.id, item_id("itm_caption"));
    assert_eq!(edges[0].consumer.clip.id, item_id("itm_video"));
    assert_eq!(edges[0].parameters.mode, TrackMatteMode::Alpha);
    assert!(!edges[0].parameters.invert);

    let edge = graph
        .matte_for_consumer(&sequence_id, &item_id("itm_next"))
        .unwrap();
    assert_eq!(edge.relation_id.as_str(), "rel_matte_next");
    assert_eq!(edge.parameters.mode, TrackMatteMode::Luma);
    assert!(edge.parameters.invert);
}

#[test]
fn sidechain_edges_are_stable_traceable_and_lookupable() {
    let mut project = relation_project();
    let parameters = SidechainRelationParameters {
        threshold_db: -12.0,
        ratio: 2.0,
        attack_ms: 5.0,
        release_ms: 100.0,
        active_range: Some(range(10, 200)),
    };
    project.project.relations.push(relation(
        "rel_sidechain_next",
        RelationKind::Sidechain {
            key: RelationEndpoint::track(track_id("trk_audio")),
            target: item("itm_next"),
            parameters: parameters.clone(),
        },
    ));
    project.project.relations.reverse();
    validate(&project).unwrap();

    let sequence_id = SequenceId::new("seq_main").unwrap();
    let graph = RelationGraph::project(&project.project);
    let edges = graph.sidechains(&sequence_id);
    assert_eq!(
        sidechain_ids(&edges),
        ["rel_sidechain", "rel_sidechain_next"]
    );
    assert_eq!(edges[0].sequence_id, &sequence_id);
    assert!(matches!(
        edges[0].key,
        RelationSignal::Track(track) if track.id == track_id("trk_audio")
    ));
    assert_eq!(edges[0].target.clip.id, item_id("itm_video"));
    assert_eq!(*edges[0].parameters, sidechain_parameters());

    let edge = graph
        .sidechain_for_target(&sequence_id, &item_id("itm_next"))
        .unwrap();
    assert_eq!(edge.relation_id.as_str(), "rel_sidechain_next");
    assert_eq!(*edge.parameters, parameters);
}

#[test]
fn dangling_matte_endpoints_do_not_partially_resolve() {
    for producer in [true, false] {
        let mut project = relation_project();
        let RelationKind::Matte {
            producer: source,
            consumer,
            ..
        } = &mut relation_mut(&mut project, "rel_matte").kind
        else {
            unreachable!()
        };
        if producer {
            *source = item("itm_missing");
        } else {
            *consumer = item("itm_missing");
        }
        assert_matte_absent(&project);
    }
}

#[test]
fn dangling_sidechain_endpoints_do_not_partially_resolve() {
    for key in [
        RelationEndpoint::track(track_id("trk_missing")),
        RelationEndpoint::bus(BusId::new("bus_missing").unwrap()),
    ] {
        let mut project = relation_project();
        let RelationKind::Sidechain { key: source, .. } =
            &mut relation_mut(&mut project, "rel_sidechain").kind
        else {
            unreachable!()
        };
        *source = key;
        assert_sidechain_absent(&project);
    }

    let mut project = relation_project();
    let RelationKind::Sidechain { target, .. } =
        &mut relation_mut(&mut project, "rel_sidechain").kind
    else {
        unreachable!()
    };
    *target = item("itm_missing");
    assert_sidechain_absent(&project);
}

#[test]
fn sidechain_bus_expands_all_routed_tracks_in_sequence_order() {
    let mut project = relation_project();
    let bus_id = BusId::new("bus_dialogue").unwrap();
    let sequence = &mut project.project.sequences[0];
    sequence.tracks[1].routing = TrackRouting::AudioBus {
        bus_id: bus_id.clone(),
    };
    let mut second = sequence.tracks[1].clone();
    second.id = track_id("trk_audio_alt");
    second.order = 15;
    second.clips[0].id = item_id("itm_audio_alt");
    second.routing = TrackRouting::AudioBus {
        bus_id: bus_id.clone(),
    };
    sequence.tracks.push(second);
    let RelationKind::Sidechain { key, .. } = &mut relation_mut(&mut project, "rel_sidechain").kind
    else {
        unreachable!()
    };
    *key = RelationEndpoint::bus(bus_id.clone());
    validate(&project).unwrap();

    let sequence_id = SequenceId::new("seq_main").unwrap();
    let graph = RelationGraph::project(&project.project);
    let edge = graph
        .sidechain_for_target(&sequence_id, &item_id("itm_video"))
        .unwrap();
    let RelationSignal::Bus {
        bus_id: actual,
        tracks,
    } = edge.key
    else {
        panic!("sidechain key must resolve to a bus")
    };
    assert_eq!(actual, &bus_id);
    let track_ids: Vec<_> = tracks.iter().map(|track| track.id.as_str()).collect();
    assert_eq!(track_ids, ["trk_audio", "trk_audio_alt"]);
}

fn matte_ids<'a>(edges: &'a [MatteEdge<'_>]) -> Vec<&'a str> {
    edges.iter().map(|edge| edge.relation_id.as_str()).collect()
}

fn sidechain_ids<'a>(edges: &'a [SidechainEdge<'_>]) -> Vec<&'a str> {
    edges.iter().map(|edge| edge.relation_id.as_str()).collect()
}

fn assert_matte_absent(project: &ProjectEnvelope) {
    let relation = relation_by_id(project, "rel_matte");
    let graph = RelationGraph::project(&project.project);
    assert!(graph.matte(relation).is_none());
    assert!(graph.mattes(&relation.sequence_id).is_empty());
}

fn assert_sidechain_absent(project: &ProjectEnvelope) {
    let relation = relation_by_id(project, "rel_sidechain");
    let graph = RelationGraph::project(&project.project);
    assert!(graph.sidechain(relation).is_none());
    assert!(graph.sidechains(&relation.sequence_id).is_empty());
}
