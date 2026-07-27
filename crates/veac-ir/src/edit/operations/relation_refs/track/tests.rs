use super::*;
use crate::test_support::{sample_project, time};

fn relation(id: &str, kind: RelationKind) -> Relation {
    Relation {
        id: RelationId::new(id).expect("relation id"),
        sequence_id: SequenceId::new("seq_relation_refs").expect("sequence id"),
        kind,
    }
}

#[test]
fn detects_track_references_in_every_relation_kind() {
    let target = TrackId::new("trk_target").expect("track id");
    let bus = BusId::new("bus_target").expect("bus id");
    let first = ItemId::new("itm_first").expect("item id");
    let second = ItemId::new("itm_second").expect("item id");
    let items = BTreeSet::from([first.clone(), second.clone()]);
    let target_endpoint = RelationEndpoint::item(first);
    let other_endpoint = RelationEndpoint::item(second);
    let relations = [
        relation(
            "rel_transition",
            RelationKind::Transition {
                from: target_endpoint.clone(),
                to: other_endpoint.clone(),
                transition: Transition {
                    kind: TransitionKind::Dissolve,
                    duration: time(120),
                    alignment: TransitionAlignment::Centered,
                },
            },
        ),
        relation(
            "rel_matte",
            RelationKind::Matte {
                producer: target_endpoint.clone(),
                consumer: other_endpoint.clone(),
                parameters: MatteRelationParameters {
                    mode: TrackMatteMode::Alpha,
                    invert: false,
                },
            },
        ),
        relation(
            "rel_sidechain",
            RelationKind::Sidechain {
                key: target_endpoint.clone(),
                target: other_endpoint.clone(),
                parameters: SidechainRelationParameters {
                    threshold_db: -18.0,
                    ratio: 4.0,
                    attack_ms: 10.0,
                    release_ms: 120.0,
                    active_range: None,
                },
            },
        ),
        relation(
            "rel_group",
            RelationKind::Group {
                members: vec![other_endpoint.clone(), target_endpoint.clone()],
            },
        ),
        relation(
            "rel_av_link",
            RelationKind::AvLink {
                video: other_endpoint,
                audio: vec![target_endpoint],
            },
        ),
    ];

    assert!(relations
        .iter()
        .all(|relation| { relation_uses(&relation.kind, &items, &target, Some(&bus)) }));
}

#[test]
fn prunes_sidechains_when_their_audio_bus_disappears() {
    let mut envelope = sample_project();
    let sequence_id = envelope.project.sequences[0].id.clone();
    let track_id = envelope.project.sequences[0].tracks[0].id.clone();
    let item_id = envelope.project.sequences[0].tracks[0].clips[0].id.clone();
    let bus_id = BusId::new("bus_disappeared").expect("bus id");
    envelope.project.relations.push(relation(
        "rel_bus_sidechain",
        RelationKind::Sidechain {
            key: RelationEndpoint::bus(bus_id.clone()),
            target: RelationEndpoint::item(item_id),
            parameters: SidechainRelationParameters {
                threshold_db: -18.0,
                ratio: 4.0,
                attack_ms: 10.0,
                release_ms: 120.0,
                active_range: None,
            },
        },
    ));
    envelope.project.relations[0].sequence_id = sequence_id.clone();

    let mut changed = ChangeSet::default();
    prune_disappeared_bus(
        &mut envelope.project,
        &sequence_id,
        &bus_id,
        &track_id,
        &mut changed,
    )
    .expect("prune relation");
    assert!(envelope.project.relations.is_empty());
}

#[test]
fn keeps_shared_buses_and_detects_the_last_audio_bus_track() {
    let mut envelope = sample_project();
    let sequence_id = envelope.project.sequences[0].id.clone();
    let track_id = envelope.project.sequences[0].tracks[0].id.clone();
    let bus_id = BusId::new("bus_shared").expect("bus id");
    let initial_track_count = envelope.project.sequences[0].tracks.len();
    envelope.project.sequences[0].tracks[0].kind = TrackKind::Audio;
    envelope.project.sequences[0].tracks[0].routing = TrackRouting::AudioBus {
        bus_id: bus_id.clone(),
    };
    let mut shared = envelope.project.sequences[0].tracks[0].clone();
    shared.id = TrackId::new("trk_shared_bus").expect("track id");
    shared.order = 1;
    envelope.project.sequences[0].tracks.push(shared);

    let mut changed = ChangeSet::default();
    prune_disappeared_bus(
        &mut envelope.project,
        &sequence_id,
        &bus_id,
        &track_id,
        &mut changed,
    )
    .expect("shared bus remains");

    envelope.project.sequences[0].tracks.pop();
    remove_track(&mut envelope.project, &sequence_id, &track_id, &mut changed)
        .expect("remove final bus track");
    assert_eq!(
        envelope.project.sequences[0].tracks.len(),
        initial_track_count
    );
}
