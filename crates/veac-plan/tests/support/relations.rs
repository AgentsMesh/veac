use veac_plan::canonical::*;

use super::{
    audio_properties, generated_clip, media_clip, project, range, time, track, visual_properties,
};

pub fn add_transition(
    envelope: &mut ProjectEnvelope,
    sequence: &str,
    from: &str,
    to: &str,
    transition: Transition,
) {
    let index = envelope.project.relations.len();
    envelope.project.relations.push(Relation {
        id: RelationId::new(format!("rel_transition_{index}")).unwrap(),
        sequence_id: SequenceId::new(sequence).unwrap(),
        kind: RelationKind::Transition {
            from: RelationEndpoint::item(ItemId::new(from).unwrap()),
            to: RelationEndpoint::item(ItemId::new(to).unwrap()),
            transition,
        },
    });
}

pub fn add_matte(
    envelope: &mut ProjectEnvelope,
    id: &str,
    sequence: &str,
    producer: &str,
    consumer: &str,
    parameters: MatteRelationParameters,
) {
    envelope.project.relations.push(Relation {
        id: RelationId::new(id).unwrap(),
        sequence_id: SequenceId::new(sequence).unwrap(),
        kind: RelationKind::Matte {
            producer: RelationEndpoint::item(ItemId::new(producer).unwrap()),
            consumer: RelationEndpoint::item(ItemId::new(consumer).unwrap()),
            parameters,
        },
    });
}

pub fn add_sidechain(
    envelope: &mut ProjectEnvelope,
    id: &str,
    sequence: &str,
    key: RelationEndpoint,
    target: &str,
    parameters: SidechainRelationParameters,
) {
    envelope.project.relations.push(Relation {
        id: RelationId::new(id).unwrap(),
        sequence_id: SequenceId::new(sequence).unwrap(),
        kind: RelationKind::Sidechain {
            key,
            target: RelationEndpoint::item(ItemId::new(target).unwrap()),
            parameters,
        },
    });
}

pub fn relation_project() -> ProjectEnvelope {
    let mut envelope = project();
    envelope.project.render_configs[0]
        .video_deliverable_mut()
        .unwrap()
        .audio = Some(AudioOutput {
        codec: AudioCodec::Aac,
        sample_rate: 48_000,
        channels: 2,
    });
    let sequence = &mut envelope.project.sequences[0];
    let mut next = {
        let target = &mut sequence.tracks[0].clips[0];
        target.visual = Some(visual_properties());
        target.audio = Some(audio_properties());
        target.clone()
    };
    next.id = ItemId::new("itm_second").unwrap();
    next.record_range = range(600, 600);
    sequence.tracks[0].clips.push(next);

    for (item_id, track_id, start, order) in [
        ("itm_matte_a", "trk_matte_a", 0, 5),
        ("itm_matte_b", "trk_matte_b", 600, 6),
    ] {
        let mut matte = generated_clip(item_id, Generator::Transparent, start);
        matte.record_range.duration = time(600);
        matte.visual = Some(visual_properties());
        sequence
            .tracks
            .push(track(track_id, TrackKind::Visual, order, vec![matte]));
    }
    let mut key = media_clip("itm_key", "med_video", 0);
    key.audio = Some(audio_properties());
    let mut key_track = track("trk_key", TrackKind::Audio, 10, vec![key]);
    key_track.routing = TrackRouting::AudioBus {
        bus_id: BusId::new("bus_dialogue").unwrap(),
    };
    sequence.tracks.push(key_track);
    envelope
}
