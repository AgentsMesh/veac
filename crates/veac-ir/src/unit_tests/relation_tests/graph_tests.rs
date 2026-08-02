use super::*;

#[test]
fn graph_exposes_sequence_and_endpoint_membership_queries() {
    let envelope = crate::test_support::sample_project();
    let sequence = &envelope.project.sequences[0];
    let item_id = sequence.tracks[0].clips[0].id.clone();
    let graph = RelationGraph::project(&envelope.project);

    assert_eq!(
        graph.sequence(&sequence.id).expect("sequence").id,
        sequence.id
    );
    assert!(graph.contains_endpoint(&sequence.id, &RelationEndpoint::item(item_id),));
    assert!(graph
        .sequence(&SequenceId::new("seq_missing").expect("sequence id"))
        .is_none());
}

#[test]
fn graph_sorts_groups_and_ignores_nonitem_av_video_endpoints() {
    let mut envelope = crate::test_support::sample_project();
    let sequence_id = envelope.project.sequences[0].id.clone();
    let item_id = envelope.project.sequences[0].tracks[0].clips[0].id.clone();
    let track_id = envelope.project.sequences[0].tracks[0].id.clone();
    for id in ["rel_group_b", "rel_group_a"] {
        envelope.project.relations.push(Relation {
            id: RelationId::new(id).expect("relation id"),
            sequence_id: sequence_id.clone(),
            kind: RelationKind::Group {
                members: vec![RelationEndpoint::item(item_id.clone())],
            },
        });
    }
    envelope.project.relations.push(Relation {
        id: RelationId::new("rel_track_av").expect("relation id"),
        sequence_id: sequence_id.clone(),
        kind: RelationKind::AvLink {
            video: RelationEndpoint::track(track_id),
            audio: vec![RelationEndpoint::item(item_id.clone())],
        },
    });

    let graph = RelationGraph::project(&envelope.project);
    let group = graph
        .group_for_item(&sequence_id, &item_id)
        .expect("group edge");
    assert_eq!(group.relation_id.as_str(), "rel_group_a");
    assert!(graph.av_link_for_item(&sequence_id, &item_id).is_none());
}

#[test]
fn graph_sorts_multiple_av_links_by_relation_id() {
    let mut envelope = crate::test_support::sample_project();
    let sequence = &envelope.project.sequences[0];
    let sequence_id = sequence.id.clone();
    let video_id = sequence.tracks[0].clips[0].id.clone();
    let caption_id = sequence.tracks[1].clips[0].id.clone();
    for id in ["rel_av_b", "rel_av_a"] {
        envelope.project.relations.push(Relation {
            id: RelationId::new(id).unwrap(),
            sequence_id: sequence_id.clone(),
            kind: RelationKind::AvLink {
                video: RelationEndpoint::item(video_id.clone()),
                audio: vec![RelationEndpoint::item(caption_id.clone())],
            },
        });
    }

    let graph = RelationGraph::project(&envelope.project);
    let ids: Vec<_> = graph
        .av_links(&sequence_id)
        .iter()
        .map(|edge| edge.relation_id.as_str())
        .collect();
    assert_eq!(ids, ["rel_av_a", "rel_av_b"]);
}

#[test]
fn graph_resolves_scoped_endpoints_and_traceable_edges() {
    let mut project = relation_project();
    let bus = BusId::new("bus_dialogue").unwrap();
    project.project.sequences[0].tracks[1].routing = TrackRouting::AudioBus {
        bus_id: bus.clone(),
    };
    let sequence_id = SequenceId::new("seq_main").unwrap();
    let graph = RelationGraph::project(&project.project);
    let scope = graph.scope(&sequence_id).unwrap();
    assert!(scope.contains(&item("itm_video")));
    assert!(scope.contains(&RelationEndpoint::track(track_id("trk_audio"))));
    assert!(scope.contains(&RelationEndpoint::bus(bus)));

    let edge = graph
        .transition_from(&sequence_id, &item_id("itm_video"))
        .unwrap();
    assert_eq!(edge.relation_id.as_str(), "rel_transition");
    assert_eq!(edge.from.position, 0);
    assert_eq!(edge.to.position, 1);
}

#[test]
fn graph_orders_edges_by_item_position_then_relation_id() {
    let mut project = relation_project();
    let sequence = &mut project.project.sequences[0];
    let mut third = sequence.tracks[0].clips[1].clone();
    third.id = item_id("itm_third");
    third.record_range = range(1200, 600);
    sequence.tracks[0].clips.push(third);
    project.project.relations.push(relation(
        "rel_first_lexically",
        RelationKind::Transition {
            from: item("itm_next"),
            to: item("itm_third"),
            transition: Transition {
                kind: TransitionKind::Dissolve,
                duration: time(60),
                alignment: TransitionAlignment::Centered,
            },
        },
    ));
    project.project.relations.reverse();
    let graph = RelationGraph::project(&project.project);
    let edges = graph.transitions(
        &SequenceId::new("seq_main").unwrap(),
        &track_id("trk_video"),
    );
    let ids: Vec<_> = edges.iter().map(|edge| edge.relation_id.as_str()).collect();
    assert_eq!(ids, ["rel_transition", "rel_first_lexically"]);
}

#[test]
fn dangling_edges_do_not_partially_resolve() {
    let mut project = relation_project();
    {
        let relation = relation_mut(&mut project, "rel_transition");
        let RelationKind::Transition { to, .. } = &mut relation.kind else {
            unreachable!()
        };
        *to = item("itm_missing");
    }
    let graph = RelationGraph::project(&project.project);
    let relation = project
        .project
        .relations
        .iter()
        .find(|value| value.id.as_str() == "rel_transition")
        .unwrap();
    assert!(graph.transition(relation).is_none());
}

#[test]
fn validator_rejects_nonadjacent_and_duplicate_outgoing_edges() {
    let mut nonadjacent = relation_project();
    let mut third = nonadjacent.project.sequences[0].tracks[0].clips[1].clone();
    third.id = item_id("itm_third");
    third.record_range = range(1200, 600);
    nonadjacent.project.sequences[0].tracks[0].clips.push(third);
    let relation = relation_mut(&mut nonadjacent, "rel_transition");
    let RelationKind::Transition { to, .. } = &mut relation.kind else {
        unreachable!()
    };
    *to = item("itm_third");
    assert_code(&nonadjacent, "RELATION_CONTEXT");

    let mut duplicate = relation_project();
    let mut second = relation_mut(&mut duplicate, "rel_transition").clone();
    second.id = RelationId::new("rel_transition_duplicate").unwrap();
    duplicate.project.relations.push(second);
    assert_code(&duplicate, "MULTIPLE_RELATION_PROJECTION");
}
